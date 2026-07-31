use crate::error::WebArenaIndigoApiError;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Delays before retrying a request rejected with 429 Too Many Requests.
const RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(1), Duration::from_secs(2)];

pub(crate) struct HttpClient;

/// Paces requests so consecutive calls stay under the API gateway's rate
/// limit (spike arrest: 2 requests/second, burst 1).
pub(crate) struct Throttle {
    interval: Duration,
    last: tokio::sync::Mutex<Option<Instant>>,
}

impl Throttle {
    pub(crate) fn new(interval: Duration) -> Self {
        Throttle {
            interval,
            last: tokio::sync::Mutex::new(None),
        }
    }

    async fn wait(&self) {
        if self.interval.is_zero() {
            return;
        }
        let mut last = self.last.lock().await;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            if elapsed < self.interval {
                tokio::time::sleep(self.interval - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }
}

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(Client::new)
}

fn with_auth(builder: RequestBuilder, token: &str) -> RequestBuilder {
    if token.is_empty() {
        builder
    } else {
        builder.bearer_auth(token)
    }
}

/// Sends the request, pacing through `throttle` and retrying on
/// 429 Too Many Requests.
async fn send(throttle: &Throttle, builder: RequestBuilder) -> Result<Response, reqwest::Error> {
    let mut builder = builder;
    for delay in RETRY_DELAYS {
        let Some(next) = builder.try_clone() else {
            break;
        };
        throttle.wait().await;
        let response = builder.send().await?;
        if response.status() != StatusCode::TOO_MANY_REQUESTS {
            return Ok(response);
        }
        tokio::time::sleep(delay).await;
        builder = next;
    }
    throttle.wait().await;
    builder.send().await
}

impl HttpClient {
    pub(crate) async fn get<T>(
        throttle: &Throttle,
        token: &str,
        url: &str,
    ) -> Result<T, WebArenaIndigoApiError>
    where
        T: DeserializeOwned,
    {
        let response = send(throttle, with_auth(client().get(url), token)).await;
        parse_json(response).await
    }

    pub(crate) async fn post<T, U>(
        throttle: &Throttle,
        token: &str,
        url: &str,
        params: &U,
    ) -> Result<T, WebArenaIndigoApiError>
    where
        T: DeserializeOwned,
        U: Serialize,
    {
        let response = send(throttle, with_auth(client().post(url), token).json(params)).await;
        parse_json(response).await
    }

    /// For endpoints that return a plain-text body instead of JSON.
    pub(crate) async fn post_text<U>(
        throttle: &Throttle,
        token: &str,
        url: &str,
        params: &U,
    ) -> Result<String, WebArenaIndigoApiError>
    where
        U: Serialize,
    {
        let response = send(throttle, with_auth(client().post(url), token).json(params)).await;
        read_body(response).await
    }

    pub(crate) async fn put<T, U>(
        throttle: &Throttle,
        token: &str,
        url: &str,
        params: &U,
    ) -> Result<T, WebArenaIndigoApiError>
    where
        T: DeserializeOwned,
        U: Serialize,
    {
        let response = send(throttle, with_auth(client().put(url), token).json(params)).await;
        parse_json(response).await
    }

    pub(crate) async fn delete<T>(
        throttle: &Throttle,
        token: &str,
        url: &str,
    ) -> Result<T, WebArenaIndigoApiError>
    where
        T: DeserializeOwned,
    {
        let response = send(throttle, with_auth(client().delete(url), token)).await;
        parse_json(response).await
    }

    pub(crate) async fn delete_with_body<T, U>(
        throttle: &Throttle,
        token: &str,
        url: &str,
        params: &U,
    ) -> Result<T, WebArenaIndigoApiError>
    where
        T: DeserializeOwned,
        U: Serialize,
    {
        let response = send(
            throttle,
            with_auth(client().delete(url), token).json(params),
        )
        .await;
        parse_json(response).await
    }
}

async fn read_body(
    response: Result<Response, reqwest::Error>,
) -> Result<String, WebArenaIndigoApiError> {
    let response = response.map_err(|e| WebArenaIndigoApiError::Connection(e.to_string()))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| WebArenaIndigoApiError::Connection(e.to_string()))?;
    if !status.is_success() {
        return Err(WebArenaIndigoApiError::Status(status.as_u16(), body));
    }
    Ok(body)
}

async fn parse_json<T>(
    response: Result<Response, reqwest::Error>,
) -> Result<T, WebArenaIndigoApiError>
where
    T: DeserializeOwned,
{
    let body = read_body(response).await?;
    serde_json::from_str(&body).map_err(|_| WebArenaIndigoApiError::JsonParse(body))
}
