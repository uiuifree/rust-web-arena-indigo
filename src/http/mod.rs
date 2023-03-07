use std::fmt::{Debug};
use reqwest::{Error, Response};
use serde_json::{json};
use crate::error::WebArenaIndigoApiError;


#[derive(Default, Debug)]
pub struct HttpClient {}


impl HttpClient {
    pub async fn get<T>(token: &str, url: &str,
                        // , params: Option<U>
    ) -> Result<T, WebArenaIndigoApiError>
        where
            T: for<'de> serde::Deserialize<'de>,
    // U: serde::Serialize + std::fmt::Debug
    {
        let response = reqwest::Client::new()
            .get(format!("{}", url))
            .header("Authorization", format!("Bearer {}", token));
        // if params.is_some() {
        // response = response.json(&json!(params));
        // }

        let response = response.send()
            .await;
        response_parse(response).await
    }
    pub async fn post<T, U>(token: &str, url: &str, params: U) -> Result<T, WebArenaIndigoApiError>
        where
            T: for<'de> serde::Deserialize<'de>,
            U: serde::Serialize + std::fmt::Debug
    {
        let mut response = reqwest::Client::new()
            .post(format!("{}", url));
        if !token.is_empty() {
            response = response.header("Authorization", format!("Bearer {}", token))
        }
        let response = response.json(&json!(params))
            .send()
            .await;
        response_parse(response).await
    }
    pub async fn put<T, U>(token: &str, url: &str, params: U) -> Result<T, WebArenaIndigoApiError>
        where
            T: for<'de> serde::Deserialize<'de>,
            U: serde::Serialize + std::fmt::Debug
    {
        let mut response = reqwest::Client::new()
            .put(format!("{}", url));
        if !token.is_empty() {
            response = response.header("Authorization", format!("Bearer {}", token))
        }
        let response = response.json(&json!(params))
            .send()
            .await;
        response_parse(response).await
    }
    pub async fn delete<T>(token: &str, url: &str) -> Result<T, WebArenaIndigoApiError>
        where
            T: for<'de> serde::Deserialize<'de>,
    {
        let mut response = reqwest::Client::new()
            .delete(format!("{}", url));
        if !token.is_empty() {
            response = response.header("Authorization", format!("Bearer {}", token))
        }
        let response = response
            .send()
            .await;
        response_parse(response).await
    }
}

async fn response_parse<T>(response: Result<Response, Error>) -> Result<T, WebArenaIndigoApiError>
    where
        T: for<'de> serde::Deserialize<'de>,
{
    if response.is_err() {
        return Err(WebArenaIndigoApiError::Connection(response.err().unwrap().to_string()));
    }
    let response = response.unwrap();
    let status = response.status().as_u16();
    let value = response.text().await;
    if !(200 <= status && status < 300) {
        return Err(WebArenaIndigoApiError::Status(status, value.unwrap().to_string()));
    }
    if value.is_err() {
        return Err(WebArenaIndigoApiError::JsonParse(value.unwrap().to_string()));
    }
    let value = value.unwrap();
    let parse = serde_json::from_str(value.as_str());
    if parse.is_err() {
        return Err(WebArenaIndigoApiError::JsonParse(value));
    }

    Ok(parse.unwrap())
}