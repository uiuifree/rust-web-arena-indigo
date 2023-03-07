mod error;
pub(crate) mod http;
mod instance;
mod access_token;
mod ssh;

pub use error::*;
use crate::access_token::AccessTokenApi;
use crate::instance::InstanceApi;
use crate::ssh::SshApi;

pub use ssh::{UpdateSshKeyRequest, SshKeyStatus};
pub use instance::{RequestCreateInstance, InstanceStatus};

pub struct WebArenaIndigoApi {
    client_id: String,
    client_secret: String,
    access_token: String,
    issued_at: u32,
    expires_in: u32,
}

impl WebArenaIndigoApi {
    pub fn new<T: Into<String>, U: Into<String>>(client_id: T, client_secret: U) -> WebArenaIndigoApi {
        let api = WebArenaIndigoApi {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            access_token: "".to_string(),
            issued_at: 0,
            expires_in: 0,
        };
        api
    }
    pub async fn update_access_token(&mut self) -> Result<&str, WebArenaIndigoApiError> {
        let res = AccessTokenApi::access_token_generate(self.client_id.as_str(), self.client_secret.as_str()).await?;
        self.access_token = res.access_token.to_string();
        self.issued_at = res.issued_at.to_string().parse().unwrap_or_default();
        self.expires_in = res.expires_in.to_string().parse().unwrap_or_default();
        Ok(self.access_token.as_str())
    }

    pub fn access_token(&self) -> &str {
        self.access_token.as_str()
    }
    pub fn instance(&mut self) -> InstanceApi {
        InstanceApi::new(self)
    }
    pub fn ssh(&mut self) -> SshApi {
        SshApi::new(self)
    }
}

pub(crate) trait IndigoApi<'a> {
    fn new(api: &'a WebArenaIndigoApi) -> Self;
    fn indigo(&self) -> &WebArenaIndigoApi;
}