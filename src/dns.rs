//! DNS APIs.

use crate::http::HttpClient;
use crate::serde_util::{u32_as_string, u32_lenient};
use crate::{WebArenaIndigoApi, WebArenaIndigoApiError};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct DnsApi<'a> {
    indigo: &'a WebArenaIndigoApi,
}

impl<'a> DnsApi<'a> {
    pub(crate) fn new(api: &'a WebArenaIndigoApi) -> Self {
        DnsApi { indigo: api }
    }

    /// `POST /webarenaIndigo/v1/dns/registerdomain`
    pub async fn register_domain(
        &self,
        domain_name: &str,
    ) -> Result<RegisterDomainResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/dns/registerdomain"),
            &json!({ "domain_name": domain_name }),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/dns/getdomainlist`
    pub async fn domain_list(&self) -> Result<Vec<DnsDomain>, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/dns/getdomainlist"),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/dns/getdomainrecord/{id}`
    pub async fn retrieve_domain(
        &self,
        domain_id: u32,
    ) -> Result<DnsDomain, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/dns/getdomainrecord/{}",
                domain_id
            )),
        )
        .await
    }

    /// `DELETE /webarenaIndigo/v1/dns/canceldomain/{id}`
    pub async fn cancel_domain(
        &self,
        domain_id: u32,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        HttpClient::delete(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/dns/canceldomain/{}",
                domain_id
            )),
        )
        .await
    }

    /// `POST /webarenaIndigo/v1/dns/domainverificationtoken`
    ///
    /// Returns the verification token. The documentation shows a plain-text
    /// response, but the live API returns a JSON-encoded string; both forms
    /// are accepted.
    pub async fn create_domain_verification_token(
        &self,
        zone_id: u32,
        verify_type: DnsVerifyType,
    ) -> Result<String, WebArenaIndigoApiError> {
        let body = HttpClient::post_text(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/dns/domainverificationtoken"),
            &json!({
                "zoneid": zone_id.to_string(),
                "verifytype": verify_type,
            }),
        )
        .await?;
        Ok(serde_json::from_str::<String>(&body).unwrap_or(body))
    }

    /// `DELETE /webarenaIndigo/v1/dns/canceldomainverification/{id}`
    pub async fn cancel_domain_verification(
        &self,
        domain_id: u32,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        HttpClient::delete(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/dns/canceldomainverification/{}",
                domain_id
            )),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/dns/domainverification/{id}`
    pub async fn verify_domain(
        &self,
        domain_id: u32,
    ) -> Result<DomainVerificationResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/dns/domainverification/{}",
                domain_id
            )),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/dns/editdomain/{id}`
    pub async fn domain_records(
        &self,
        domain_id: u32,
    ) -> Result<DomainRecordsResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint(&format!("/webarenaIndigo/v1/dns/editdomain/{}", domain_id)),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/dns/getttl`
    pub async fn ttl_list(&self) -> Result<Vec<DnsTtl>, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/dns/getttl"),
        )
        .await
    }

    /// `POST /webarenaIndigo/v1/dns/addARecord`
    pub async fn add_a_record(
        &self,
        request: ARecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.post_record("/webarenaIndigo/v1/dns/addARecord", &request)
            .await
    }

    /// `PUT /webarenaIndigo/v1/dns/updateARecord`
    pub async fn update_a_record(
        &self,
        request: ARecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.put_record("/webarenaIndigo/v1/dns/updateARecord", &request)
            .await
    }

    /// `POST /webarenaIndigo/v1/dns/addAAAARecord`
    pub async fn add_aaaa_record(
        &self,
        request: AaaaRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.post_record("/webarenaIndigo/v1/dns/addAAAARecord", &request)
            .await
    }

    /// `PUT /webarenaIndigo/v1/dns/updateAAAARecord`
    pub async fn update_aaaa_record(
        &self,
        request: AaaaRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.put_record("/webarenaIndigo/v1/dns/updateAAAARecord", &request)
            .await
    }

    /// `POST /webarenaIndigo/v1/dns/addCNAMERecord`
    pub async fn add_cname_record(
        &self,
        request: CnameRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.post_record("/webarenaIndigo/v1/dns/addCNAMERecord", &request)
            .await
    }

    /// `PUT /webarenaIndigo/v1/dns/updateCNAMERecord`
    pub async fn update_cname_record(
        &self,
        request: CnameRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.put_record("/webarenaIndigo/v1/dns/updateCNAMERecord", &request)
            .await
    }

    /// `POST /webarenaIndigo/v1/dns/addMXRecord`
    pub async fn add_mx_record(
        &self,
        request: MxRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.post_record("/webarenaIndigo/v1/dns/addMXRecord", &request)
            .await
    }

    /// `PUT /webarenaIndigo/v1/dns/updateMXRecord`
    pub async fn update_mx_record(
        &self,
        request: MxRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.put_record("/webarenaIndigo/v1/dns/updateMXRecord", &request)
            .await
    }

    /// `POST /webarenaIndigo/v1/dns/addTXTRecord`
    pub async fn add_txt_record(
        &self,
        request: TxtRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.post_record("/webarenaIndigo/v1/dns/addTXTRecord", &request)
            .await
    }

    /// `PUT /webarenaIndigo/v1/dns/updateTXTRecord`
    pub async fn update_txt_record(
        &self,
        request: TxtRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.put_record("/webarenaIndigo/v1/dns/updateTXTRecord", &request)
            .await
    }

    /// `POST /webarenaIndigo/v1/dns/addNSRecord`
    pub async fn add_ns_record(
        &self,
        request: NsRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.post_record("/webarenaIndigo/v1/dns/addNSRecord", &request)
            .await
    }

    /// `PUT /webarenaIndigo/v1/dns/updateNSRecord`
    pub async fn update_ns_record(
        &self,
        request: NsRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.put_record("/webarenaIndigo/v1/dns/updateNSRecord", &request)
            .await
    }

    /// `POST /webarenaIndigo/v1/dns/addSRVRecord`
    pub async fn add_srv_record(
        &self,
        request: SrvRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.post_record("/webarenaIndigo/v1/dns/addSRVRecord", &request)
            .await
    }

    /// `PUT /webarenaIndigo/v1/dns/updateSRVRecord`
    pub async fn update_srv_record(
        &self,
        request: SrvRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        self.put_record("/webarenaIndigo/v1/dns/updateSRVRecord", &request)
            .await
    }

    /// `DELETE /webarenaIndigo/v1/dns/deleteNSRecord`
    pub async fn destroy_record(
        &self,
        request: DeleteRecordRequest,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        HttpClient::delete_with_body(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/dns/deleteNSRecord"),
            &request,
        )
        .await
    }

    async fn post_record<U: Serialize>(
        &self,
        path: &str,
        request: &U,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(path),
            request,
        )
        .await
    }

    async fn put_record<U: Serialize>(
        &self,
        path: &str,
        request: &U,
    ) -> Result<DnsOperationResponse, WebArenaIndigoApiError> {
        HttpClient::put(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(path),
            request,
        )
        .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDomainResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sucessCode")]
    pub success_code: String,
    #[serde(rename = "zoneId")]
    pub zone_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsDomain {
    pub id: u32,
    pub zone: String,
    pub zone_id: u32,
    pub verify_status: String,
    pub verify_type: String,
    pub verify_token: String,
    pub user_id: u32,
    pub service_id: String,
    pub deleted: u32,
    pub created_at: String,
    pub verified_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsOperationResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sucessCode")]
    pub success_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainVerificationResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sucessCode")]
    pub success_code: String,
    /// The API returns this as either a number or a string.
    #[serde(rename = "DNSZoneId", deserialize_with = "u32_lenient")]
    pub dns_zone_id: u32,
    #[serde(rename = "type")]
    pub verify_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecordsResponse {
    pub zone_records: Vec<ZoneRecord>,
}

/// The shape of `value` depends on the record type (e.g. an array of
/// addresses for A records, SOA attributes for SOA records).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneRecord {
    pub value: serde_json::Value,
    pub ttl: u32,
    pub name: String,
    pub class: String,
    #[serde(rename = "type")]
    pub record_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsTtl {
    pub id: u32,
    pub name: String,
    pub value: u32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum DnsVerifyType {
    #[serde(rename = "TXT")]
    Txt,
    #[serde(rename = "NS")]
    Ns,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum DnsRecordType {
    A,
    #[serde(rename = "AAAA")]
    Aaaa,
    #[serde(rename = "CNAME")]
    Cname,
    #[serde(rename = "MX")]
    Mx,
    #[serde(rename = "TXT")]
    Txt,
    #[serde(rename = "NS")]
    Ns,
    #[serde(rename = "SRV")]
    Srv,
}

#[derive(Debug, Clone, Serialize)]
pub struct ARecordRequest {
    #[serde(serialize_with = "u32_as_string")]
    pub zone_id: u32,
    pub domain_name: String,
    pub host_name: String,
    pub address: Vec<String>,
    #[serde(serialize_with = "u32_as_string")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct AaaaRecordRequest {
    #[serde(serialize_with = "u32_as_string")]
    pub zone_id: u32,
    pub domain_name: String,
    pub host_name: String,
    pub address: String,
    #[serde(serialize_with = "u32_as_string")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CnameRecordRequest {
    #[serde(serialize_with = "u32_as_string")]
    pub zone_id: u32,
    pub domain_name: String,
    pub host_name: String,
    pub host_name_alias: String,
    #[serde(serialize_with = "u32_as_string")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MxRecordRequest {
    #[serde(serialize_with = "u32_as_string")]
    pub zone_id: u32,
    pub domain_name: String,
    pub host_name: String,
    pub mail_server: String,
    #[serde(serialize_with = "u32_as_string")]
    pub priority: u32,
    #[serde(serialize_with = "u32_as_string")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TxtRecordRequest {
    #[serde(serialize_with = "u32_as_string")]
    pub zone_id: u32,
    pub domain_name: String,
    pub host_name: String,
    pub txt_value: String,
    #[serde(serialize_with = "u32_as_string")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NsRecordRequest {
    #[serde(serialize_with = "u32_as_string")]
    pub zone_id: u32,
    pub domain_name: String,
    pub host_name: String,
    pub to_server: Vec<String>,
    #[serde(serialize_with = "u32_as_string")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SrvRecordRequest {
    #[serde(serialize_with = "u32_as_string")]
    pub zone_id: u32,
    pub domain_name: String,
    pub host_name: String,
    pub priority: u32,
    pub target: String,
    pub weight: u32,
    pub port: u32,
    #[serde(serialize_with = "u32_as_string")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteRecordRequest {
    #[serde(serialize_with = "u32_as_string")]
    pub zone_id: u32,
    pub domain_name: String,
    pub host_name: String,
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
}
