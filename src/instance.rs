//! Instance (VM) APIs.

use crate::http::HttpClient;
use crate::serde_util::opt_string_lenient;
use crate::{WebArenaIndigoApi, WebArenaIndigoApiError};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct InstanceApi<'a> {
    indigo: &'a WebArenaIndigoApi,
}

impl<'a> InstanceApi<'a> {
    pub(crate) fn new(api: &'a WebArenaIndigoApi) -> Self {
        InstanceApi { indigo: api }
    }

    /// `GET /webarenaIndigo/v1/vm/instancetypes`
    pub async fn instance_type_list(
        &self,
    ) -> Result<InstanceTypeListResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/vm/instancetypes"),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/vm/getregion`
    pub async fn region_list(
        &self,
        instance_type_id: u32,
    ) -> Result<RegionListResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/vm/getregion?instanceTypeId={}",
                instance_type_id
            )),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/vm/oslist`
    pub async fn os_list(
        &self,
        instance_type_id: u32,
    ) -> Result<OsListResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/vm/oslist?instanceTypeId={}",
                instance_type_id
            )),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/vm/getinstancespec`
    pub async fn instance_specification(
        &self,
        instance_type_id: u32,
    ) -> Result<InstanceSpecificationResponse, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/vm/getinstancespec?instanceTypeId={}",
                instance_type_id
            )),
        )
        .await
    }

    /// `POST /webarenaIndigo/v1/vm/createinstance`
    pub async fn create_instance(
        &self,
        request: CreateInstanceRequest,
    ) -> Result<CreateInstanceResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/vm/createinstance"),
            &request,
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/vm/getinstancelist`
    pub async fn instance_list(&self) -> Result<Vec<Instance>, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/vm/getinstancelist"),
        )
        .await
    }

    /// `POST /webarenaIndigo/v1/vm/instance/statusupdate`
    ///
    /// [`InstanceStatus::Destroy`] deletes the instance.
    pub async fn update_instance_status(
        &self,
        instance_id: u32,
        status: InstanceStatus,
    ) -> Result<UpdateInstanceStatusResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/vm/instance/statusupdate"),
            &json!({
                "instanceId": instance_id,
                "status": status,
            }),
        )
        .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceTypeListResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "instanceTypes")]
    pub instance_types: Vec<InstanceType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceType {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionListResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "regionlist")]
    pub region_list: Vec<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub id: u32,
    pub name: String,
    pub use_possible_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsListResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "osCategory")]
    pub os_category: Vec<OsCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsCategory {
    pub id: u32,
    pub name: String,
    pub logo: String,
    #[serde(rename = "osLists")]
    pub os_lists: Vec<OsList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsList {
    pub id: u32,
    /// The live API sends `categoryid` although the documentation says
    /// `categoryId`; both are accepted.
    #[serde(rename = "categoryid", alias = "categoryId")]
    pub category_id: u32,
    /// Not documented, but returned by the live API (e.g. `"Ubuntu"`).
    pub code: Option<String>,
    pub name: String,
    #[serde(rename = "viewname")]
    pub view_name: String,
    #[serde(rename = "instancetype_id")]
    pub instance_type_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSpecificationResponse {
    pub success: bool,
    pub total: u32,
    #[serde(rename = "speclist")]
    pub spec_list: Vec<InstanceSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSpec {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub use_possible_date: String,
    #[serde(rename = "instancetype_id")]
    pub instance_type_id: u32,
    pub created_at: String,
    pub updated_at: String,
    pub instance_type: InstanceType,
}

/// Request body for [`InstanceApi::create_instance`]. One variant per
/// documented boot method.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum CreateInstanceRequest {
    /// Boot a Linux instance with an SSH key.
    Standard {
        #[serde(rename = "sshKeyId")]
        ssh_key_id: u32,
        #[serde(rename = "regionId")]
        region_id: u32,
        #[serde(rename = "osId")]
        os_id: u32,
        #[serde(rename = "instancePlan")]
        instance_plan: u32,
        #[serde(rename = "instanceName")]
        instance_name: String,
    },
    /// Boot a Windows instance with an administrator password.
    Windows {
        #[serde(rename = "winPassword")]
        win_password: String,
        #[serde(rename = "regionId")]
        region_id: u32,
        #[serde(rename = "osId")]
        os_id: u32,
        #[serde(rename = "instancePlan")]
        instance_plan: u32,
        #[serde(rename = "instanceName")]
        instance_name: String,
    },
    /// Boot from an image imported from a URL.
    Import {
        #[serde(rename = "importUrl")]
        import_url: String,
        #[serde(rename = "regionId")]
        region_id: u32,
        #[serde(rename = "osId")]
        os_id: u32,
        #[serde(rename = "instancePlan")]
        instance_plan: u32,
        #[serde(rename = "instanceName")]
        instance_name: String,
    },
    /// Boot from an existing snapshot.
    FromSnapshot {
        #[serde(rename = "sshKeyId")]
        ssh_key_id: u32,
        #[serde(rename = "snapshotId")]
        snapshot_id: u32,
        #[serde(rename = "instancePlan")]
        instance_plan: u32,
        #[serde(rename = "instanceName")]
        instance_name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInstanceResponse {
    pub success: bool,
    pub message: String,
    pub vms: Instance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: u32,
    pub instance_name: String,
    pub set_no: u32,
    /// The API returns this as either a number or a string.
    #[serde(default, deserialize_with = "opt_string_lenient")]
    pub vps_kind: Option<String>,
    pub sequence_id: u32,
    pub user_id: u32,
    pub service_id: String,
    pub status: String,
    #[serde(rename = "sshkey_id")]
    pub ssh_key_id: u32,
    pub start_date: Option<InstanceDate>,
    pub host_id: u32,
    pub plan: String,
    pub disk_point: u32,
    #[serde(rename = "memsize")]
    pub mem_size: u32,
    pub cpus: u32,
    pub os_id: u32,
    #[serde(rename = "otherstatus")]
    pub other_status: u32,
    pub uuid: Option<String>,
    #[serde(rename = "uidgid")]
    pub uid_gid: u32,
    pub vnc_port: u32,
    pub vnc_passwd: String,
    #[serde(rename = "arpaname")]
    pub arpa_name: Option<String>,
    #[serde(rename = "arpadate")]
    pub arpa_date: u32,
    pub ipaddress: Option<String>,
    pub secondary_ip: Option<String>,
    pub macaddress: Option<String>,
    pub ipaddress_type: Option<String>,
    pub status_change_date: Option<InstanceDate>,
    pub updated_at: Option<InstanceDate>,
    pub vm_revert: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceDate {
    pub date: String,
    pub timezone_type: u32,
    pub timezone: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum InstanceStatus {
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "stop")]
    Stop,
    #[serde(rename = "forcestop")]
    ForceStop,
    #[serde(rename = "reset")]
    Reset,
    /// Deletes the instance.
    #[serde(rename = "destroy")]
    Destroy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInstanceStatusResponse {
    pub success: bool,
    pub message: String,
    #[serde(rename = "sucessCode")]
    pub success_code: String,
    #[serde(rename = "instanceStatus")]
    pub instance_status: String,
}
