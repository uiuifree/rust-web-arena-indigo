use serde_derive::{Deserialize, Serialize};
use serde_json::{json};
use crate::http::HttpClient;
use crate::{IndigoApi, WebArenaIndigoApi, WebArenaIndigoApiError};


pub struct SshApi<'a> {
    indigo: &'a WebArenaIndigoApi,
}

impl<'a> IndigoApi<'a> for SshApi<'a> {
    fn new(api: &'a WebArenaIndigoApi) -> Self {
        SshApi {
            indigo: api
        }
    }

    fn indigo(&self) -> &WebArenaIndigoApi {
        self.indigo
    }
}


impl<'a> SshApi<'a> {
    pub async fn ssh_key_list(&self) -> Result<SshKeyListResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::get(
                self.indigo().access_token(),
                "https://api.customer.jp/webarenaIndigo/v1/vm/sshkey",
            ).await?
        )
    }
    pub async fn active_ssh_key_list(&self) -> Result<ActiveSshKeyListResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::get(
                self.indigo().access_token(),
                "https://api.customer.jp/webarenaIndigo/v1/vm/sshkey/active/status",
            ).await?
        )
    }
    pub async fn create_ssh_key(&self, name: &str, public_key: &str) -> Result<CreateSshKeyResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::post(
                self.indigo().access_token(),
                "https://api.customer.jp/webarenaIndigo/v1/vm/sshkey",
                json!(
                    {
                        "sshName":name,
                    "sshKey":public_key,}
                ),
            ).await?
        )
    }
    pub async fn retrieve_ssh_key(&self, ssh_key_id: u32) -> Result<RetrieveSshKeyResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::get(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/sshkey/{}", ssh_key_id).as_str(),
            ).await?
        )
    }
    pub async fn update_ssh_key(&self, ssh_key_id: u32, request: UpdateSshKeyRequest) -> Result<UpdateSshKeyResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::put(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/sshkey/{}", ssh_key_id).as_str(),
                request,
            ).await?
        )
    }
    pub async fn destroy_ssh_key(&self, ssh_key_id: u32) -> Result<DestroySshKeyResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::delete(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/sshkey/{}", ssh_key_id).as_str(),
            ).await?
        )
    }
}


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SshKeyListResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "sshkeys")]
    pub ssh_keys: Vec<SshKey>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
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

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ActiveSshKeyListResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "sshkeys")]
    pub ssh_keys: Vec<SshKey>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CreateSshKeyResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sshKey")]
    pub ssh_key: SshKey,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RetrieveSshKeyResponse {
    pub success: bool,
    #[serde(rename = "sshKey")]
    pub ssh_key: Vec<SshKey>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UpdateSshKeyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UpdateSshKeyRequest {
    #[serde(rename = "sshName")]
    pub ssh_name: String,
    #[serde(rename = "sshKey")]
    pub ssh_key: String,
    #[serde(rename = "sshKeyStatus")]
    pub ssh_key_status: SshKeyStatus,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub enum SshKeyStatus {
    #[default]
    ACTIVE,
    DEACTIVE,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DestroySshKeyResponse {
    pub success: bool,
    pub message: String,
}