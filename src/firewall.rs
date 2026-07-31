//! Firewall APIs.

use crate::http::HttpClient;
use crate::serde_util::{u32_as_string, u32_lenient, u32_vec_as_strings};
use crate::{WebArenaIndigoApi, WebArenaIndigoApiError};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Firewall APIs, obtained via [`WebArenaIndigoApi::firewall`](crate::WebArenaIndigoApi::firewall).
pub struct FirewallApi<'a> {
    indigo: &'a WebArenaIndigoApi,
}

impl<'a> FirewallApi<'a> {
    pub(crate) fn new(api: &'a WebArenaIndigoApi) -> Self {
        FirewallApi { indigo: api }
    }

    /// `POST /webarenaIndigo/v1/nw/createfirewall`
    pub async fn create_firewall(
        &self,
        request: CreateFirewallRequest,
    ) -> Result<FirewallTemplateResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/nw/createfirewall"),
            &request,
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/nw/getfirewalllist`
    pub async fn firewall_list(&self) -> Result<Vec<Firewall>, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/nw/getfirewalllist"),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/nw/gettemplate/{id}`
    pub async fn retrieve_firewall(
        &self,
        firewall_id: u32,
    ) -> Result<Vec<FirewallTemplateRule>, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/nw/gettemplate/{}",
                firewall_id
            )),
        )
        .await
    }

    /// `PUT /webarenaIndigo/v1/nw/updatefirewall`
    pub async fn update_firewall(
        &self,
        request: UpdateFirewallRequest,
    ) -> Result<FirewallTemplateResponse, WebArenaIndigoApiError> {
        HttpClient::put(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/nw/updatefirewall"),
            &request,
        )
        .await
    }

    /// `POST /webarenaIndigo/v1/nw/assign`
    pub async fn assign_firewall(
        &self,
        firewall_id: u32,
        instance_id: u32,
    ) -> Result<FirewallOperationResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/nw/assign"),
            &json!({
                "templateid": firewall_id.to_string(),
                "instanceid": instance_id.to_string(),
            }),
        )
        .await
    }

    /// `DELETE /webarenaIndigo/v1/nw/deletefirewall/{id}`
    pub async fn destroy_firewall(
        &self,
        firewall_id: u32,
    ) -> Result<FirewallOperationResponse, WebArenaIndigoApiError> {
        HttpClient::delete(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/nw/deletefirewall/{}",
                firewall_id
            )),
        )
        .await
    }
}

/// Request body for [`FirewallApi::create_firewall`].
#[derive(Debug, Clone, Serialize)]
pub struct CreateFirewallRequest {
    pub name: String,
    pub inbound: Vec<FirewallRule>,
    pub outbound: Vec<FirewallRule>,
    #[serde(serialize_with = "u32_vec_as_strings")]
    pub instances: Vec<u32>,
}

/// Request body for [`FirewallApi::update_firewall`].
#[derive(Debug, Clone, Serialize)]
pub struct UpdateFirewallRequest {
    #[serde(rename = "templateid", serialize_with = "u32_as_string")]
    pub firewall_id: u32,
    pub name: String,
    pub inbound: Vec<FirewallRule>,
    pub outbound: Vec<FirewallRule>,
    #[serde(serialize_with = "u32_vec_as_strings")]
    pub instances: Vec<u32>,
}

/// A single allow rule of a firewall template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    #[serde(rename = "type")]
    pub rule_type: String,
    pub protocol: String,
    pub port: String,
    pub source: String,
}

/// Response of [`FirewallApi::create_firewall`] and [`FirewallApi::update_firewall`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallTemplateResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sucessCode")]
    pub success_code: String,
    /// The live API returns this as a number on create and as a string on
    /// update; both are accepted.
    #[serde(rename = "firewallId", deserialize_with = "u32_lenient")]
    pub firewall_id: u32,
}

/// A firewall template entry, as returned by [`FirewallApi::firewall_list`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Firewall {
    pub id: u32,
    pub service_id: String,
    pub user_id: u32,
    pub name: String,
    pub status: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// A rule of a template, as returned by [`FirewallApi::retrieve_firewall`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallTemplateRule {
    pub id: u32,
    pub name: String,
    pub direction: String,
    #[serde(rename = "type")]
    pub rule_type: String,
    pub protocol: String,
    pub port: String,
    pub source: String,
}

/// Response of [`FirewallApi::assign_firewall`] and [`FirewallApi::destroy_firewall`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallOperationResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sucessCode")]
    pub success_code: String,
}
