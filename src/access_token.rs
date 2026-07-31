use crate::WebArenaIndigoApiError;
use crate::http::{HttpClient, Throttle};
use crate::serde_util::u64_lenient;
use serde::{Deserialize, Serialize};

pub(crate) struct AccessTokenApi;

impl AccessTokenApi {
    /// `POST /oauth/v1/accesstokens`
    pub(crate) async fn access_token_generate(
        throttle: &Throttle,
        base_url: &str,
        client_id: &str,
        client_secret: &str,
    ) -> Result<AccessTokenGenerateResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            throttle,
            "",
            &format!("{}/oauth/v1/accesstokens", base_url),
            &AccessTokenGenerateRequest {
                grant_type: "client_credentials".to_string(),
                client_id: client_id.to_string(),
                client_secret: client_secret.to_string(),
                code: String::new(),
            },
        )
        .await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AccessTokenGenerateResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Lifetime in seconds. The API returns this as a string.
    #[serde(rename = "expiresIn", deserialize_with = "u64_lenient")]
    pub expires_in: u64,
    /// Issue time in milliseconds since the UNIX epoch. Returned as a string.
    #[serde(rename = "issuedAt", deserialize_with = "u64_lenient")]
    pub issued_at: u64,
}

#[derive(Debug, Clone, Serialize)]
struct AccessTokenGenerateRequest {
    #[serde(rename = "grantType")]
    grant_type: String,
    #[serde(rename = "clientId")]
    client_id: String,
    #[serde(rename = "clientSecret")]
    client_secret: String,
    code: String,
}
