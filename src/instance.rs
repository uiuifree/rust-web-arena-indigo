use serde_derive::{Deserialize, Serialize};
use serde_json::{json};
use crate::http::HttpClient;
use crate::{IndigoApi, WebArenaIndigoApi, WebArenaIndigoApiError};


pub struct InstanceApi<'a> {
    indigo: &'a WebArenaIndigoApi,
}

impl<'a> IndigoApi<'a> for InstanceApi<'a> {
    fn new(api: &'a WebArenaIndigoApi) -> Self {
        InstanceApi {
            indigo: api
        }
    }

    fn indigo(&self) -> &WebArenaIndigoApi {
        self.indigo
    }
}


impl<'a> InstanceApi<'a> {
    pub async fn instance_type_list(&self) -> Result<InstanceTypeListResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::get(
                self.indigo().access_token(),
                "https://api.customer.jp/webarenaIndigo/v1/vm/instancetypes",
            ).await?
        )
    }
    pub async fn region_list(&self, instance_type_id: u32) -> Result<GetRegionListResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::get(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/getregion?instanceTypeId={}", instance_type_id).as_str(),
            ).await?
        )
    }
    pub async fn os_list(&self, instance_type_id: u32) -> Result<GetOsListResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::get(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/oslist?instanceTypeId={}", instance_type_id).as_str(),
            ).await?
        )
    }
    pub async fn instance_specification(&self, instance_type_id: u32) -> Result<GetInstanceSpecification, WebArenaIndigoApiError> {
        Ok(
            HttpClient::get(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/getinstancespec?instanceTypeId={}", instance_type_id).as_str(),
            ).await?
        )
    }
    pub async fn create_instance(&self, request: RequestCreateInstance) -> Result<CreateInstanceResponse, WebArenaIndigoApiError> {
        Ok(
            HttpClient::post(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/createinstance", ).as_str(),
                request,
            ).await?
        )
    }
    pub async fn get_instance_list(&self) -> Result<Vec<Vms>, WebArenaIndigoApiError> {
        Ok(
            HttpClient::get(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/getinstancelist", ).as_str(),
            ).await?
        )
    }
    pub async fn update_instance_status(&self, instance_id: u32, status: InstanceStatus) -> Result<UpdateInstanceStatusResponse, WebArenaIndigoApiError> {
        println!("{}",json!({
                    "instanceId":instance_id,
                    "status":status,
                }).to_string());

        Ok(
            HttpClient::post(
                self.indigo().access_token(),
                format!("https://api.customer.jp/webarenaIndigo/v1/vm/instance/statusupdate", ).as_str(),
                json!({
                    "instanceId":instance_id,
                    "status":status,
                }),
            ).await?
        )
    }
}


#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InstanceTypeListResponse {
    success: bool,
    total: u32,
    #[serde(rename = "instanceTypes")]
    instance_types: Vec<InstanceType>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InstanceType {
    id: u32,
    name: String,
    display_name: String,
    created_at: String,
    updated_at: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GetRegionListResponse {
    success: bool,
    total: u32,
    #[serde(rename = "regionlist")]
    region_list: Vec<Region>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Region {
    id: u32,
    name: String,
    use_possible_date: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GetOsListResponse {
    success: bool,
    total: u32,
    #[serde(rename = "osCategory")]
    os_category: Vec<OsCategory>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct OsCategory {
    id: u32,
    name: String,
    logo: String,
    #[serde(rename = "osLists")]
    os_lists: Vec<OsList>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct OsList {
    id: u32,
    #[serde(rename = "categoryId")]
    category_id: u32,
    name: String,
    #[serde(rename = "viewname")]
    view_name: String,
    #[serde(rename = "instancetype_id")]
    instance_type_id: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GetInstanceSpecification {
    success: bool,
    total: u32,
    #[serde(rename = "speclist")]
    spec_list: Vec<SpecList>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct SpecList {
    id: u32,
    name: String,
    description: String,
    use_possible_date: String,
    #[serde(rename = "instancetype_id")]
    instance_type_id: u32,
    created_at: String,
    updated_at: String,
    instance_type: InstanceType,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RequestCreateInstance {
    #[serde(rename = "sshKeyId")]
    pub ssh_key_id: u32,
    #[serde(rename = "regionId")]
    pub region_id: u32,
    #[serde(rename = "osId")]
    pub os_id: u32,
    #[serde(rename = "instancePlan")]
    pub instance_plan: u32,
    #[serde(rename = "instanceName")]
    pub instance_name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct CreateInstanceResponse {
    success: bool,
    message: String,
    vms: Vms,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Vms {
    pub id: u32,
    pub instance_name: String,
    pub set_no: u32,
    // pub // vps_kind: String, // number and string
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
    pub ipaddress:Option<String>,
    pub secondary_ip:Option<String>,
    pub macaddress:Option<String>,
    pub ipaddress_type:Option<String>,
    pub status_change_date: Option<InstanceDate>,
    pub updated_at: Option<InstanceDate>,
    pub vm_revert: u32,

}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct InstanceDate {
    date: String,
    timezone_type: u32,
    timezone: String,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub enum InstanceStatus {
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "stop")]
    #[default]
    Stop,
    #[serde(rename = "forcestop")]
    ForceStop,
    #[serde(rename = "reset")]
    Reset,
    #[serde(rename = "destroy")]
    Destroy,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct UpdateInstanceStatusResponse{
    success:bool,
    message:String,
    #[serde(rename = "sucessCode")]
    sucess_code:String,
    #[serde(rename = "instanceStatus")]
    instance_status:String,
}