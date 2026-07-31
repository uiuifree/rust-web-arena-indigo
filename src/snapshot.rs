//! Snapshot APIs.

use crate::http::HttpClient;
use crate::serde_util::{u32_lenient, u64_lenient};
use crate::{WebArenaIndigoApi, WebArenaIndigoApiError};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub struct SnapshotApi<'a> {
    indigo: &'a WebArenaIndigoApi,
}

impl<'a> SnapshotApi<'a> {
    pub(crate) fn new(api: &'a WebArenaIndigoApi) -> Self {
        SnapshotApi { indigo: api }
    }

    /// `POST /webarenaIndigo/v1/disk/takesnapshot`
    pub async fn take_snapshot(
        &self,
        name: &str,
        instance_id: u32,
        slot_num: u32,
    ) -> Result<SnapshotStatusResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint("/webarenaIndigo/v1/disk/takesnapshot"),
            &json!({
                "name": name,
                "instanceid": instance_id,
                "slotnum": slot_num.to_string(),
            }),
        )
        .await
    }

    /// `GET /webarenaIndigo/v1/disk/snapshotlist/{instance_id}`
    pub async fn snapshot_list(
        &self,
        instance_id: u32,
    ) -> Result<Vec<Snapshot>, WebArenaIndigoApiError> {
        HttpClient::get(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/disk/snapshotlist/{}",
                instance_id
            )),
        )
        .await
    }

    /// `POST /webarenaIndigo/v1/disk/retakesnapshot`
    pub async fn retake_snapshot(
        &self,
        instance_id: u32,
        snapshot_id: u32,
    ) -> Result<SnapshotStatusResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/disk/retakesnapshot"),
            &json!({
                "instanceid": instance_id,
                "snapshotid": snapshot_id.to_string(),
            }),
        )
        .await
    }

    /// `POST /webarenaIndigo/v1/disk/restoresnapshot`
    pub async fn restore_snapshot(
        &self,
        instance_id: u32,
        snapshot_id: u32,
    ) -> Result<SnapshotStatusResponse, WebArenaIndigoApiError> {
        HttpClient::post(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self
                .indigo
                .endpoint("/webarenaIndigo/v1/disk/restoresnapshot"),
            &json!({
                "instanceid": instance_id.to_string(),
                "snapshotid": snapshot_id.to_string(),
            }),
        )
        .await
    }

    /// `DELETE /webarenaIndigo/v1/disk/deletesnapshot/{id}`
    pub async fn destroy_snapshot(
        &self,
        snapshot_id: u32,
    ) -> Result<SnapshotStatusResponse, WebArenaIndigoApiError> {
        HttpClient::delete(
            self.indigo.throttle(),
            self.indigo.access_token(),
            &self.indigo.endpoint(&format!(
                "/webarenaIndigo/v1/disk/deletesnapshot/{}",
                snapshot_id
            )),
        )
        .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotStatusResponse {
    #[serde(rename = "STATUS")]
    pub status: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: u32,
    pub name: String,
    pub service_id: String,
    /// The API returns this as either a number or a string.
    #[serde(deserialize_with = "u32_lenient")]
    pub user_id: u32,
    pub disk_id: u32,
    pub volume: u32,
    pub slot_number: u32,
    pub status: String,
    /// The API returns this as either a number or a string.
    #[serde(deserialize_with = "u64_lenient")]
    pub size: u64,
    pub deleted: u32,
    pub completed_timestamp: String,
    pub deleted_timestamp: String,
}
