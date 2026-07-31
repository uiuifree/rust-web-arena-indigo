//! Unofficial async Rust client for the [WebARENA Indigo](https://web.arena.ne.jp/indigo/)
//! VPS API by NTTPC Communications (not affiliated with or endorsed by NTTPC).
//!
//! Covers every endpoint of the official REST API: instances (VMs), SSH keys, API keys,
//! firewalls, snapshots and DNS.
//!
//! API reference: <https://indigo.arena.ne.jp/userapi/>
//!
//! ```no_run
//! use web_arena_indigo::WebArenaIndigoApi;
//!
//! # async fn run() -> Result<(), web_arena_indigo::WebArenaIndigoApiError> {
//! let mut api = WebArenaIndigoApi::new("CLIENT_ID", "CLIENT_SECRET");
//! api.update_access_token().await?;
//! for vm in api.instance().instance_list().await? {
//!     println!("{} {} {:?}", vm.instance_name, vm.status, vm.ipaddress);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! Each API group is exposed via [`WebArenaIndigoApi::instance`],
//! [`WebArenaIndigoApi::ssh`], [`WebArenaIndigoApi::api_key`],
//! [`WebArenaIndigoApi::firewall`], [`WebArenaIndigoApi::snapshot`] and
//! [`WebArenaIndigoApi::dns`].

mod access_token;
pub mod api_key;
pub mod dns;
mod error;
pub mod firewall;
pub(crate) mod http;
pub mod instance;
pub(crate) mod serde_util;
pub mod snapshot;
pub mod ssh;

pub use error::WebArenaIndigoApiError;
pub use instance::{CreateInstanceRequest, InstanceStatus};
pub use ssh::{SshKeyStatus, UpdateSshKeyRequest};

use crate::access_token::AccessTokenApi;
use crate::api_key::ApiKeyApi;
use crate::dns::DnsApi;
use crate::firewall::FirewallApi;
use crate::http::Throttle;
use crate::instance::InstanceApi;
use crate::snapshot::SnapshotApi;
use crate::ssh::SshApi;
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.customer.jp";

/// Default pacing between requests. The Indigo API gateway enforces a spike
/// arrest of 2 requests/second with a burst of 1.
const DEFAULT_REQUEST_INTERVAL: Duration = Duration::from_millis(600);

/// Entry point of the client. Holds the credentials and the current access token.
pub struct WebArenaIndigoApi {
    client_id: String,
    client_secret: String,
    access_token: String,
    issued_at: u64,
    expires_in: u64,
    base_url: String,
    throttle: Throttle,
}

impl WebArenaIndigoApi {
    /// Creates a client from the client ID / secret issued in the Indigo
    /// control panel.
    pub fn new<T: Into<String>, U: Into<String>>(
        client_id: T,
        client_secret: U,
    ) -> WebArenaIndigoApi {
        WebArenaIndigoApi {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            access_token: String::new(),
            issued_at: 0,
            expires_in: 0,
            base_url: DEFAULT_BASE_URL.to_string(),
            throttle: Throttle::new(DEFAULT_REQUEST_INTERVAL),
        }
    }

    /// Overrides the API endpoint, e.g. for a proxy or a mock server in tests.
    pub fn with_base_url<T: Into<String>>(mut self, base_url: T) -> WebArenaIndigoApi {
        self.base_url = base_url.into();
        self
    }

    /// Reuses an access token issued earlier instead of calling
    /// [`update_access_token`](Self::update_access_token).
    pub fn with_access_token<T: Into<String>>(mut self, access_token: T) -> WebArenaIndigoApi {
        self.access_token = access_token.into();
        self
    }

    /// Overrides the minimum interval between requests. The default (600 ms)
    /// stays under the Indigo API gateway's rate limit of 2 requests/second
    /// with a burst of 1. Pass [`Duration::ZERO`] to disable pacing, e.g.
    /// against a mock server.
    pub fn with_request_interval(mut self, interval: Duration) -> WebArenaIndigoApi {
        self.throttle = Throttle::new(interval);
        self
    }

    /// Issues a new access token (`POST /oauth/v1/accesstokens`) and stores it
    /// for subsequent calls.
    pub async fn update_access_token(&mut self) -> Result<&str, WebArenaIndigoApiError> {
        let res = AccessTokenApi::access_token_generate(
            &self.throttle,
            &self.base_url,
            &self.client_id,
            &self.client_secret,
        )
        .await?;
        self.access_token = res.access_token;
        self.issued_at = res.issued_at;
        self.expires_in = res.expires_in;
        Ok(&self.access_token)
    }

    /// The access token currently held by the client (empty until issued).
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Issue time of the current token, in milliseconds since the UNIX epoch.
    pub fn issued_at(&self) -> u64 {
        self.issued_at
    }

    /// Lifetime of the current token, in seconds.
    pub fn expires_in(&self) -> u64 {
        self.expires_in
    }

    pub(crate) fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub(crate) fn throttle(&self) -> &Throttle {
        &self.throttle
    }

    /// Instance (VM) APIs.
    pub fn instance(&self) -> InstanceApi<'_> {
        InstanceApi::new(self)
    }
    /// SSH key APIs.
    pub fn ssh(&self) -> SshApi<'_> {
        SshApi::new(self)
    }
    /// API key APIs.
    pub fn api_key(&self) -> ApiKeyApi<'_> {
        ApiKeyApi::new(self)
    }
    /// Firewall APIs.
    pub fn firewall(&self) -> FirewallApi<'_> {
        FirewallApi::new(self)
    }
    /// Snapshot APIs.
    pub fn snapshot(&self) -> SnapshotApi<'_> {
        SnapshotApi::new(self)
    }
    /// DNS APIs.
    pub fn dns(&self) -> DnsApi<'_> {
        DnsApi::new(self)
    }
}
