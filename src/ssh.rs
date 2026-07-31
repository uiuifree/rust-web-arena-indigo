//! SSH key APIs.

use crate::http::HttpClient;
use crate::{WebArenaIndigoApi, WebArenaIndigoApiError};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// SSH key APIs, obtained via [`WebArenaIndigoApi::ssh`](crate::WebArenaIndigoApi::ssh).
pub struct SshApi<'a> {
    indigo: &'a WebArenaIndigoApi,
}

impl<'a> SshApi<'a> {
    pub(crate) fn new(api: &'a WebArenaIndigoApi) -> Self {
        SshApi { indigo: api }
    }

    /// `GET /webarenaIndigo/v1/vm/sshkey`
    pub async fn ssh_key_list(&self) -> Result<SshKeyListResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/vm/sshkey"),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/vm/sshkey/active/status`
    pub async fn active_ssh_key_list(
        &self,
    ) -> Result<ActiveSshKeyListResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/vm/sshkey/active/status"),
        )
        .await
    }

    /// `POST /webarenaIndigo/v1/vm/sshkey`
    pub async fn create_ssh_key(
        &self,
        name: &str,
        public_key: &str,
    ) -> Result<CreateSshKeyResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/vm/sshkey"),
            &json!({
                "sshName": name,
                "sshKey": public_key,
            }),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/vm/sshkey/{id}`
    pub async fn retrieve_ssh_key(
        &self,
        ssh_key_id: u32,
    ) -> Result<RetrieveSshKeyResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint(&format!("/webarenaIndigo/v1/vm/sshkey/{}", ssh_key_id)),
        )
        .await
    }

    /// `PUT /webarenaIndigo/v1/vm/sshkey/{id}`
    pub async fn update_ssh_key(
        &self,
        ssh_key_id: u32,
        request: UpdateSshKeyRequest,
    ) -> Result<UpdateSshKeyResponse, WebArenaIndigoApiError> {
        HttpClient::put(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint(&format!("/webarenaIndigo/v1/vm/sshkey/{}", ssh_key_id)),
            &request,
        )
        .await
    }

    /// `DELETE /webarenaIndigo/v1/vm/sshkey/{id}`
    pub async fn destroy_ssh_key(
        &self,
        ssh_key_id: u32,
    ) -> Result<DestroySshKeyResponse, WebArenaIndigoApiError> {
        HttpClient::delete(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint(&format!("/webarenaIndigo/v1/vm/sshkey/{}", ssh_key_id)),
        )
        .await
    }
}

/// Response of [`SshApi::ssh_key_list`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyListResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "sshkeys")]
    pub ssh_keys: Vec<SshKey>,
}

/// A registered SSH public key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
    pub id: u32,
    pub service_id: String,
    pub user_id: u32,
    pub name: String,
    #[serde(rename = "sshkey")]
    pub ssh_key: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Response of [`SshApi::active_ssh_key_list`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSshKeyListResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "sshkeys")]
    pub ssh_keys: Vec<SshKey>,
}

/// Response of [`SshApi::create_ssh_key`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSshKeyResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sshKey")]
    pub ssh_key: SshKey,
}

/// Response of [`SshApi::retrieve_ssh_key`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveSshKeyResponse {
    pub success: bool,
    #[serde(rename = "sshKey")]
    pub ssh_key: Vec<SshKey>,
}

/// Response of [`SshApi::update_ssh_key`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSshKeyResponse {
    pub success: bool,
    pub message: String,
}

/// Request body for [`SshApi::update_ssh_key`].
#[derive(Debug, Clone, Serialize)]
pub struct UpdateSshKeyRequest {
    #[serde(rename = "sshName")]
    pub ssh_name: String,
    #[serde(rename = "sshKey")]
    pub ssh_key: String,
    #[serde(rename = "sshKeyStatus")]
    pub ssh_key_status: SshKeyStatus,
}

/// SSH key status, serialized as `ACTIVE` / `DEACTIVE`.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub enum SshKeyStatus {
    #[default]
    #[serde(rename = "ACTIVE")]
    Active,
    #[serde(rename = "DEACTIVE")]
    Deactive,
}

/// Response of [`SshApi::destroy_ssh_key`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestroySshKeyResponse {
    pub success: bool,
    pub message: String,
}
