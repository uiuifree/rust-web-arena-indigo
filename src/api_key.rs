//! API key APIs.

use crate::http::HttpClient;
use crate::{WebArenaIndigoApi, WebArenaIndigoApiError};
use serde::{Deserialize, Serialize};

pub struct ApiKeyApi<'a> {
    indigo: &'a WebArenaIndigoApi,
}

impl<'a> ApiKeyApi<'a> {
    pub(crate) fn new(api: &'a WebArenaIndigoApi) -> Self {
        ApiKeyApi { indigo: api }
    }

    /// `GET /webarenaIndigo/v1/auth/create/apikey`
    pub async fn create_api_key(&self) -> Result<CreateApiKeyResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/auth/create/apikey"),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/auth/apikey`
    pub async fn api_key_list(&self) -> Result<ApiKeyListResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/auth/apikey"),
        )
        .await
    }

    /// `DELETE /webarenaIndigo/v1/auth/apikey/{id}`
    pub async fn destroy_api_key(
        &self,
        api_key_id: u32,
    ) -> Result<DestroyApiKeyResponse, WebArenaIndigoApiError> {
        HttpClient::delete(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint(&format!("/webarenaIndigo/v1/auth/apikey/{}", api_key_id)),
        )
        .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "apiSecret")]
    pub api_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyListResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "accesstokens")]
    pub api_keys: Vec<ApiKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: u32,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestroyApiKeyResponse {
    pub success: bool,
    pub message: String,
}
