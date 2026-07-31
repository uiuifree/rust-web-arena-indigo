//! Offline tests for every API method, using a local mock server.
//! Request bodies and response fixtures follow the official examples at
//! <https://indigo.arena.ne.jp/userapi/>.

use mockito::{Matcher, Server, ServerGuard};
use serde_json::json;
use std::time::Duration;
use web_arena_indigo::dns::{
    ARecordRequest, AaaaRecordRequest, CnameRecordRequest, DeleteRecordRequest, DnsRecordType,
    DnsVerifyType, MxRecordRequest, NsRecordRequest, SrvRecordRequest, TxtRecordRequest,
};
use web_arena_indigo::firewall::{CreateFirewallRequest, FirewallRule, UpdateFirewallRequest};
use web_arena_indigo::{
    CreateInstanceRequest, InstanceStatus, SshKeyStatus, UpdateSshKeyRequest, WebArenaIndigoApi,
    WebArenaIndigoApiError,
};

const TOKEN: &str = "test-token";

async fn server_and_api() -> (ServerGuard, WebArenaIndigoApi) {
    let server = Server::new_async().await;
    let api = WebArenaIndigoApi::new("test-id", "test-secret")
        .with_base_url(server.url())
        .with_access_token(TOKEN)
        .with_request_interval(Duration::ZERO);
    (server, api)
}

fn vms_json() -> serde_json::Value {
    json!({
        "id": 476020,
        "instance_name": "test",
        "set_no": 1,
        "sequence_id": 1,
        "user_id": 100,
        "service_id": "wsi-000001",
        "status": "UNUSED",
        "sshkey_id": 15532,
        "start_date": {"date": "2022-07-14 07:57:56.000000", "timezone_type": 3, "timezone": "Asia/Tokyo"},
        "host_id": 2,
        "plan": "2CR2GB",
        "disk_point": 12,
        "memsize": 2,
        "cpus": 2,
        "os_id": 13,
        "otherstatus": 10,
        "uuid": "0c951dc1-4c6f-4d4b-a5fa-4be27a94b040",
        "uidgid": 100000,
        "vnc_port": 5901,
        "vnc_passwd": "vncpass",
        "arpaname": null,
        "arpadate": 0,
        "ipaddress": "192.0.2.1",
        "secondary_ip": null,
        "macaddress": "02:00:00:00:00:01",
        "ipaddress_type": "global",
        "status_change_date": null,
        "updated_at": null,
        "vm_revert": 0
    })
}

fn ssh_key_json() -> serde_json::Value {
    json!({
        "id": 15532,
        "service_id": "wsi-000001",
        "user_id": 100,
        "name": "test",
        "sshkey": "ssh-rsa AAA",
        "status": "ACTIVE",
        "created_at": "2018-11-01 00:00:00",
        "updated_at": "2018-11-01 00:00:00"
    })
}

#[tokio::test]
async fn access_token() {
    let (mut server, _) = server_and_api().await;
    let mock = server
        .mock("POST", "/oauth/v1/accesstokens")
        .match_body(Matcher::Json(json!({
            "grantType": "client_credentials",
            "clientId": "test-id",
            "clientSecret": "test-secret",
            "code": ""
        })))
        .with_body(
            json!({
                "accessToken": "tok123",
                "tokenType": "BearerToken",
                "expiresIn": "3599",
                "scope": "",
                "issuedAt": "1657781876000"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let mut api = WebArenaIndigoApi::new("test-id", "test-secret").with_base_url(server.url());
    let token = api.update_access_token().await.unwrap().to_string();
    assert_eq!(token, "tok123");
    assert_eq!(api.access_token(), "tok123");
    assert_eq!(api.issued_at(), 1657781876000);
    assert_eq!(api.expires_in(), 3599);
    mock.assert_async().await;
}

#[tokio::test]
async fn instance_api() {
    let (mut server, api) = server_and_api().await;
    let auth = format!("Bearer {}", TOKEN);

    let instance_type = json!({
        "id": 1,
        "name": "instanceV1",
        "display_name": "KVM",
        "created_at": "2018-07-31 06:56:36",
        "updated_at": "2018-07-31 06:56:36"
    });

    let types = server
        .mock("GET", "/webarenaIndigo/v1/vm/instancetypes")
        .match_header("authorization", auth.as_str())
        .with_body(
            json!({"success": true, "total": 1, "instanceTypes": [instance_type]}).to_string(),
        )
        .create_async()
        .await;
    let res = api.instance().instance_type_list().await.unwrap();
    assert!(res.success);
    assert_eq!(res.instance_types[0].id, 1);
    types.assert_async().await;

    let regions = server
        .mock("GET", "/webarenaIndigo/v1/vm/getregion")
        .match_query(Matcher::UrlEncoded("instanceTypeId".into(), "1".into()))
        .match_header("authorization", auth.as_str())
        .with_body(
            json!({
                "success": true,
                "total": 1,
                "regionlist": [{"id": 1, "name": "Tokyo1", "use_possible_date": "2018-07-31 06:56:36"}]
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.instance().region_list(1).await.unwrap();
    assert_eq!(res.region_list[0].name, "Tokyo1");
    regions.assert_async().await;

    let os_list = server
        .mock("GET", "/webarenaIndigo/v1/vm/oslist")
        .match_query(Matcher::UrlEncoded("instanceTypeId".into(), "1".into()))
        .match_header("authorization", auth.as_str())
        .with_body(
            json!({
                "success": true,
                "total": 1,
                "osCategory": [{
                    "id": 1,
                    "name": "Ubuntu",
                    "logo": "ubuntu.png",
                    "osLists": [{"id": 13, "categoryid": 1, "code": "Ubuntu", "name": "Ubuntu22.04", "viewname": "Ubuntu 22.04", "instancetype_id": 1}]
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.instance().os_list(1).await.unwrap();
    assert_eq!(res.os_category[0].os_lists[0].id, 13);
    os_list.assert_async().await;

    let spec = server
        .mock("GET", "/webarenaIndigo/v1/vm/getinstancespec")
        .match_query(Matcher::UrlEncoded("instanceTypeId".into(), "1".into()))
        .match_header("authorization", auth.as_str())
        .with_body(
            json!({
                "success": true,
                "total": 1,
                "speclist": [{
                    "id": 1,
                    "name": "t1.nano",
                    "description": "1 CPU / 1GB RAM",
                    "use_possible_date": "2018-07-31 06:56:36",
                    "instancetype_id": 1,
                    "created_at": "2018-07-31 06:56:36",
                    "updated_at": "2018-07-31 06:56:36",
                    "instance_type": instance_type
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.instance().instance_specification(1).await.unwrap();
    assert_eq!(res.spec_list[0].name, "t1.nano");
    spec.assert_async().await;

    let create = server
        .mock("POST", "/webarenaIndigo/v1/vm/createinstance")
        .match_header("authorization", auth.as_str())
        .match_body(Matcher::Json(json!({
            "sshKeyId": 15532,
            "regionId": 1,
            "osId": 13,
            "instancePlan": 1,
            "instanceName": "test"
        })))
        .with_body(
            json!({"success": true, "message": "Instance created successfully", "vms": vms_json()})
                .to_string(),
        )
        .create_async()
        .await;
    let res = api
        .instance()
        .create_instance(CreateInstanceRequest::Standard {
            ssh_key_id: 15532,
            region_id: 1,
            os_id: 13,
            instance_plan: 1,
            instance_name: "test".to_string(),
        })
        .await
        .unwrap();
    assert_eq!(res.vms.id, 476020);
    create.assert_async().await;

    let create_windows = server
        .mock("POST", "/webarenaIndigo/v1/vm/createinstance")
        .match_header("authorization", auth.as_str())
        .match_body(Matcher::Json(json!({
            "winPassword": "Passw0rd!",
            "regionId": 1,
            "osId": 20,
            "instancePlan": 5,
            "instanceName": "win"
        })))
        .with_body(
            json!({"success": true, "message": "Instance created successfully", "vms": vms_json()})
                .to_string(),
        )
        .create_async()
        .await;
    let res = api
        .instance()
        .create_instance(CreateInstanceRequest::Windows {
            win_password: "Passw0rd!".to_string(),
            region_id: 1,
            os_id: 20,
            instance_plan: 5,
            instance_name: "win".to_string(),
        })
        .await
        .unwrap();
    assert!(res.success);
    create_windows.assert_async().await;

    let list = server
        .mock("GET", "/webarenaIndigo/v1/vm/getinstancelist")
        .match_header("authorization", auth.as_str())
        .with_body(json!([vms_json()]).to_string())
        .create_async()
        .await;
    let res = api.instance().instance_list().await.unwrap();
    assert_eq!(res[0].ipaddress.as_deref(), Some("192.0.2.1"));
    list.assert_async().await;

    let status = server
        .mock("POST", "/webarenaIndigo/v1/vm/instance/statusupdate")
        .match_header("authorization", auth.as_str())
        .match_body(Matcher::Json(
            json!({"instanceId": 476020, "status": "stop"}),
        ))
        .with_body(
            json!({
                "success": true,
                "message": "Success",
                "sucessCode": "I20005",
                "instanceStatus": "stopping"
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api
        .instance()
        .update_instance_status(476020, InstanceStatus::Stop)
        .await
        .unwrap();
    assert_eq!(res.success_code, "I20005");
    status.assert_async().await;
}

#[tokio::test]
async fn ssh_api() {
    let (mut server, api) = server_and_api().await;

    let list = server
        .mock("GET", "/webarenaIndigo/v1/vm/sshkey")
        .with_body(json!({"success": true, "total": 1, "sshkeys": [ssh_key_json()]}).to_string())
        .create_async()
        .await;
    let res = api.ssh().ssh_key_list().await.unwrap();
    assert_eq!(res.ssh_keys[0].id, 15532);
    list.assert_async().await;

    let active = server
        .mock("GET", "/webarenaIndigo/v1/vm/sshkey/active/status")
        .with_body(json!({"success": true, "total": 1, "sshkeys": [ssh_key_json()]}).to_string())
        .create_async()
        .await;
    let res = api.ssh().active_ssh_key_list().await.unwrap();
    assert_eq!(res.ssh_keys[0].status, "ACTIVE");
    active.assert_async().await;

    let create = server
        .mock("POST", "/webarenaIndigo/v1/vm/sshkey")
        .match_body(Matcher::Json(json!({"sshName": "test", "sshKey": "ssh-rsa AAA"})))
        .with_body(
            json!({"success": true, "message": "SSH key has been added successfully.", "sshKey": ssh_key_json()})
                .to_string(),
        )
        .create_async()
        .await;
    let res = api
        .ssh()
        .create_ssh_key("test", "ssh-rsa AAA")
        .await
        .unwrap();
    assert_eq!(res.ssh_key.id, 15532);
    create.assert_async().await;

    let retrieve = server
        .mock("GET", "/webarenaIndigo/v1/vm/sshkey/15532")
        .with_body(json!({"success": true, "sshKey": [ssh_key_json()]}).to_string())
        .create_async()
        .await;
    let res = api.ssh().retrieve_ssh_key(15532).await.unwrap();
    assert_eq!(res.ssh_key[0].name, "test");
    retrieve.assert_async().await;

    let update = server
        .mock("PUT", "/webarenaIndigo/v1/vm/sshkey/15532")
        .match_body(Matcher::Json(json!({
            "sshName": "test2",
            "sshKey": "ssh-rsa AAA",
            "sshKeyStatus": "DEACTIVE"
        })))
        .with_body(
            json!({"success": true, "message": "SSH key has been updated successfully."})
                .to_string(),
        )
        .create_async()
        .await;
    let res = api
        .ssh()
        .update_ssh_key(
            15532,
            UpdateSshKeyRequest {
                ssh_name: "test2".to_string(),
                ssh_key: "ssh-rsa AAA".to_string(),
                ssh_key_status: SshKeyStatus::Deactive,
            },
        )
        .await
        .unwrap();
    assert!(res.success);
    update.assert_async().await;

    let destroy = server
        .mock("DELETE", "/webarenaIndigo/v1/vm/sshkey/15532")
        .with_body(
            json!({"success": true, "message": "SSH key has been removed successfully."})
                .to_string(),
        )
        .create_async()
        .await;
    let res = api.ssh().destroy_ssh_key(15532).await.unwrap();
    assert!(res.success);
    destroy.assert_async().await;
}

#[tokio::test]
async fn api_key_api() {
    let (mut server, api) = server_and_api().await;

    let create = server
        .mock("GET", "/webarenaIndigo/v1/auth/create/apikey")
        .with_body(
            json!({"apiKey": "m70QyrbMUZWl06SfSAvRBPQO0ofsadf", "apiSecret": "LmAY0pB1xA1fas"})
                .to_string(),
        )
        .create_async()
        .await;
    let res = api.api_key().create_api_key().await.unwrap();
    assert_eq!(res.api_secret, "LmAY0pB1xA1fas");
    create.assert_async().await;

    let list = server
        .mock("GET", "/webarenaIndigo/v1/auth/apikey")
        .with_body(
            json!({
                "success": true,
                "total": 1,
                "accesstokens": [{
                    "id": 434,
                    "apiKey": "3PwzRzZyXAmBSi0NiYGQjUGpDfsadf",
                    "created_at": "2019-10-21 11:17:08"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.api_key().api_key_list().await.unwrap();
    assert_eq!(res.api_keys[0].id, 434);
    list.assert_async().await;

    let destroy = server
        .mock("DELETE", "/webarenaIndigo/v1/auth/apikey/88")
        .with_body(
            json!({"success": true, "message": "API Key is removed successfully"}).to_string(),
        )
        .create_async()
        .await;
    let res = api.api_key().destroy_api_key(88).await.unwrap();
    assert!(res.success);
    destroy.assert_async().await;
}

#[tokio::test]
async fn firewall_api() {
    let (mut server, api) = server_and_api().await;

    let rules = vec![
        FirewallRule {
            rule_type: "HTTP".to_string(),
            protocol: "TCP".to_string(),
            port: "80".to_string(),
            source: "0.0.0.0".to_string(),
        },
        FirewallRule {
            rule_type: "HTTPS".to_string(),
            protocol: "TCP".to_string(),
            port: "443".to_string(),
            source: "0.0.0.0".to_string(),
        },
    ];
    let rules_json = json!([
        {"type": "HTTP", "protocol": "TCP", "port": "80", "source": "0.0.0.0"},
        {"type": "HTTPS", "protocol": "TCP", "port": "443", "source": "0.0.0.0"}
    ]);

    let create = server
        .mock("POST", "/webarenaIndigo/v1/nw/createfirewall")
        .match_body(Matcher::Json(json!({
            "name": "Example",
            "inbound": rules_json,
            "outbound": rules_json,
            "instances": ["6", "5"]
        })))
        .with_body(
            json!({
                "success": true,
                "message": "Firewall template has been created successfully.",
                "sucessCode": "F60002",
                "firewallId": 55
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api
        .firewall()
        .create_firewall(CreateFirewallRequest {
            name: "Example".to_string(),
            inbound: rules.clone(),
            outbound: rules.clone(),
            instances: vec![6, 5],
        })
        .await
        .unwrap();
    assert_eq!(res.firewall_id, 55);
    create.assert_async().await;

    let list = server
        .mock("GET", "/webarenaIndigo/v1/nw/getfirewalllist")
        .with_body(
            json!([{
                "id": 55,
                "service_id": "wsi-00003",
                "user_id": 6,
                "name": "Example",
                "status": 1,
                "created_at": "2018-11-14 10:35:33",
                "updated_at": "2018-11-14 10:35:33"
            }])
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.firewall().firewall_list().await.unwrap();
    assert_eq!(res[0].id, 55);
    list.assert_async().await;

    let retrieve = server
        .mock("GET", "/webarenaIndigo/v1/nw/gettemplate/55")
        .with_body(
            json!([{
                "id": 55,
                "name": "Example",
                "direction": "in",
                "type": "HTTP",
                "protocol": "TCP",
                "port": "80",
                "source": "0.0.0.0"
            }])
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.firewall().retrieve_firewall(55).await.unwrap();
    assert_eq!(res[0].direction, "in");
    retrieve.assert_async().await;

    let update = server
        .mock("PUT", "/webarenaIndigo/v1/nw/updatefirewall")
        .match_body(Matcher::Json(json!({
            "templateid": "55",
            "name": "Example",
            "inbound": rules_json,
            "outbound": rules_json,
            "instances": ["6", "5"]
        })))
        .with_body(
            // The live API returns firewallId as a string on update.
            json!({
                "success": true,
                "message": "Firewall template is updated successfully.",
                "sucessCode": "F6004",
                "firewallId": "55"
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api
        .firewall()
        .update_firewall(UpdateFirewallRequest {
            firewall_id: 55,
            name: "Example".to_string(),
            inbound: rules.clone(),
            outbound: rules,
            instances: vec![6, 5],
        })
        .await
        .unwrap();
    assert_eq!(res.success_code, "F6004");
    update.assert_async().await;

    let assign = server
        .mock("POST", "/webarenaIndigo/v1/nw/assign")
        .match_body(Matcher::Json(json!({"instanceid": "4", "templateid": "8"})))
        .with_body(
            json!({
                "success": true,
                "message": "Firewall template is assigned successfully.",
                "sucessCode": "F60003"
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.firewall().assign_firewall(8, 4).await.unwrap();
    assert_eq!(res.success_code, "F60003");
    assign.assert_async().await;

    let destroy = server
        .mock("DELETE", "/webarenaIndigo/v1/nw/deletefirewall/30")
        .with_body(
            json!({
                "success": true,
                "message": "Firewall template has been deleted successfully.",
                "sucessCode": "F6005"
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.firewall().destroy_firewall(30).await.unwrap();
    assert_eq!(res.success_code, "F6005");
    destroy.assert_async().await;
}

#[tokio::test]
async fn snapshot_api() {
    let (mut server, api) = server_and_api().await;

    let take = server
        .mock("POST", "/webarenaIndigo/v1/disk/takesnapshot")
        .match_body(Matcher::Json(
            json!({"name": "Example", "instanceid": 12, "slotnum": "0"}),
        ))
        .with_body(json!({"STATUS": 0}).to_string())
        .create_async()
        .await;
    let res = api
        .snapshot()
        .take_snapshot("Example", 12, 0)
        .await
        .unwrap();
    assert_eq!(res.status, 0);
    take.assert_async().await;

    let list = server
        .mock("GET", "/webarenaIndigo/v1/disk/snapshotlist/12")
        .with_body(
            json!([{
                "id": 3,
                "name": "Example",
                "service_id": "wsi-000401",
                "user_id": "134",
                "disk_id": 29,
                "volume": 1,
                "slot_number": 0,
                "status": "created",
                "size": "2000",
                "deleted": 0,
                "completed_timestamp": "2018-11-27 07:24:05",
                "deleted_timestamp": "0000-00-00 00:00:00"
            }])
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.snapshot().snapshot_list(12).await.unwrap();
    assert_eq!(res[0].id, 3);
    list.assert_async().await;

    let retake = server
        .mock("POST", "/webarenaIndigo/v1/disk/retakesnapshot")
        .match_body(Matcher::Json(json!({"instanceid": 12, "snapshotid": "11"})))
        .with_body(json!({"STATUS": 0}).to_string())
        .create_async()
        .await;
    let res = api.snapshot().retake_snapshot(12, 11).await.unwrap();
    assert_eq!(res.status, 0);
    retake.assert_async().await;

    let restore = server
        .mock("POST", "/webarenaIndigo/v1/disk/restoresnapshot")
        .match_body(Matcher::Json(
            json!({"instanceid": "12", "snapshotid": "12"}),
        ))
        .with_body(json!({"STATUS": 0}).to_string())
        .create_async()
        .await;
    let res = api.snapshot().restore_snapshot(12, 12).await.unwrap();
    assert_eq!(res.status, 0);
    restore.assert_async().await;

    let destroy = server
        .mock("DELETE", "/webarenaIndigo/v1/disk/deletesnapshot/10")
        .with_body(json!({"STATUS": 0}).to_string())
        .create_async()
        .await;
    let res = api.snapshot().destroy_snapshot(10).await.unwrap();
    assert_eq!(res.status, 0);
    destroy.assert_async().await;
}

#[tokio::test]
async fn dns_domain_api() {
    let (mut server, api) = server_and_api().await;

    let domain_json = json!({
        "id": 26,
        "zone": "example.com",
        "zone_id": 0,
        "verify_status": "Verification Pending",
        "verify_type": "",
        "verify_token": "",
        "user_id": 128,
        "service_id": "wsi-000403",
        "deleted": 0,
        "created_at": "2019-02-28 10:03:25",
        "verified_at": null,
        "deleted_at": null
    });

    let register = server
        .mock("POST", "/webarenaIndigo/v1/dns/registerdomain")
        .match_body(Matcher::Json(json!({"domain_name": "example.com"})))
        .with_body(
            json!({
                "success": true,
                "message": "Domain added successfully",
                "sucessCode": "D80002",
                "zoneId": 26
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.dns().register_domain("example.com").await.unwrap();
    assert_eq!(res.zone_id, 26);
    register.assert_async().await;

    let list = server
        .mock("GET", "/webarenaIndigo/v1/dns/getdomainlist")
        .with_body(json!([domain_json]).to_string())
        .create_async()
        .await;
    let res = api.dns().domain_list().await.unwrap();
    assert_eq!(res[0].zone, "example.com");
    list.assert_async().await;

    let retrieve = server
        .mock("GET", "/webarenaIndigo/v1/dns/getdomainrecord/26")
        .with_body(domain_json.to_string())
        .create_async()
        .await;
    let res = api.dns().retrieve_domain(26).await.unwrap();
    assert_eq!(res.id, 26);
    retrieve.assert_async().await;

    let cancel = server
        .mock("DELETE", "/webarenaIndigo/v1/dns/canceldomain/26")
        .with_body(
            json!({"success": true, "message": "Domain cancelled successfully.", "sucessCode": "D80003"}).to_string(),
        )
        .create_async()
        .await;
    let res = api.dns().cancel_domain(26).await.unwrap();
    assert_eq!(res.success_code, "D80003");
    cancel.assert_async().await;

    let token = server
        .mock("POST", "/webarenaIndigo/v1/dns/domainverificationtoken")
        .match_body(Matcher::Json(json!({"zoneid": "29", "verifytype": "NS"})))
        .with_body("\"o6j5brpsjbnjdsu7wzvo8ub0j05akk.customer.ne.jp\"")
        .create_async()
        .await;
    let res = api
        .dns()
        .create_domain_verification_token(29, DnsVerifyType::Ns)
        .await
        .unwrap();
    assert_eq!(res, "o6j5brpsjbnjdsu7wzvo8ub0j05akk.customer.ne.jp");
    token.assert_async().await;

    let cancel_verification = server
        .mock(
            "DELETE",
            "/webarenaIndigo/v1/dns/canceldomainverification/26",
        )
        .with_body(
            json!({
                "success": true,
                "message": "Domain verification cancelled successfully.",
                "sucessCode": "D80024"
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.dns().cancel_domain_verification(26).await.unwrap();
    assert_eq!(res.success_code, "D80024");
    cancel_verification.assert_async().await;

    let verify = server
        .mock("GET", "/webarenaIndigo/v1/dns/domainverification/43")
        .with_body(
            json!({
                "success": true,
                "message": "Domain verification completed successfully",
                "sucessCode": "D80018",
                "DNSZoneId": "102756",
                "type": "NS"
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.dns().verify_domain(43).await.unwrap();
    assert_eq!(res.dns_zone_id, 102756);
    verify.assert_async().await;

    let records = server
        .mock("GET", "/webarenaIndigo/v1/dns/editdomain/43")
        .with_body(
            json!({
                "zone_records": [{
                    "value": [{"minimum": "3600", "serial": "1551174766"}],
                    "ttl": 86400,
                    "name": "example.com",
                    "class": "IN",
                    "type": "SOA"
                }]
            })
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.dns().domain_records(43).await.unwrap();
    assert_eq!(res.zone_records[0].record_type, "SOA");
    records.assert_async().await;

    let ttl = server
        .mock("GET", "/webarenaIndigo/v1/dns/getttl")
        .with_body(
            json!([
                {"id": 1, "name": "5 Minutes", "value": 300},
                {"id": 8, "name": "1 Day", "value": 86400}
            ])
            .to_string(),
        )
        .create_async()
        .await;
    let res = api.dns().ttl_list().await.unwrap();
    assert_eq!(res[1].value, 86400);
    ttl.assert_async().await;
}

#[tokio::test]
async fn dns_record_api() {
    let (mut server, api) = server_and_api().await;

    async fn mock_record(
        server: &mut ServerGuard,
        method: &str,
        path: &str,
        body: serde_json::Value,
        code: &str,
    ) -> mockito::Mock {
        server
            .mock(method, path)
            .match_body(Matcher::Json(body))
            .with_body(json!({"success": true, "message": "ok", "sucessCode": code}).to_string())
            .create_async()
            .await
    }

    let a_body = json!({
        "zone_id": "102756",
        "domain_name": "example.com",
        "host_name": "dev",
        "address": ["192.168.0.11", "192.168.1.2"],
        "ttl": "600"
    });
    let a_request = || ARecordRequest {
        zone_id: 102756,
        domain_name: "example.com".to_string(),
        host_name: "dev".to_string(),
        address: vec!["192.168.0.11".to_string(), "192.168.1.2".to_string()],
        ttl: 600,
    };
    let mock = mock_record(
        &mut server,
        "POST",
        "/webarenaIndigo/v1/dns/addARecord",
        a_body.clone(),
        "D80005",
    )
    .await;
    assert_eq!(
        api.dns()
            .add_a_record(a_request())
            .await
            .unwrap()
            .success_code,
        "D80005"
    );
    mock.assert_async().await;
    let mock = mock_record(
        &mut server,
        "PUT",
        "/webarenaIndigo/v1/dns/updateARecord",
        a_body,
        "D80004",
    )
    .await;
    assert_eq!(
        api.dns()
            .update_a_record(a_request())
            .await
            .unwrap()
            .success_code,
        "D80004"
    );
    mock.assert_async().await;

    let aaaa_body = json!({
        "zone_id": "102756",
        "domain_name": "example.com",
        "host_name": "dev",
        "address": "FE80:0000:0000:0000:0202:B3FF:FE1E:8329",
        "ttl": "600"
    });
    let aaaa_request = || AaaaRecordRequest {
        zone_id: 102756,
        domain_name: "example.com".to_string(),
        host_name: "dev".to_string(),
        address: "FE80:0000:0000:0000:0202:B3FF:FE1E:8329".to_string(),
        ttl: 600,
    };
    let mock = mock_record(
        &mut server,
        "POST",
        "/webarenaIndigo/v1/dns/addAAAARecord",
        aaaa_body.clone(),
        "D80007",
    )
    .await;
    assert_eq!(
        api.dns()
            .add_aaaa_record(aaaa_request())
            .await
            .unwrap()
            .success_code,
        "D80007"
    );
    mock.assert_async().await;
    let mock = mock_record(
        &mut server,
        "PUT",
        "/webarenaIndigo/v1/dns/updateAAAARecord",
        aaaa_body,
        "D80008",
    )
    .await;
    assert_eq!(
        api.dns()
            .update_aaaa_record(aaaa_request())
            .await
            .unwrap()
            .success_code,
        "D80008"
    );
    mock.assert_async().await;

    let cname_body = json!({
        "zone_id": "102756",
        "domain_name": "example.com",
        "host_name": "www",
        "host_name_alias": "test.com",
        "ttl": "600"
    });
    let cname_request = || CnameRecordRequest {
        zone_id: 102756,
        domain_name: "example.com".to_string(),
        host_name: "www".to_string(),
        host_name_alias: "test.com".to_string(),
        ttl: 600,
    };
    let mock = mock_record(
        &mut server,
        "POST",
        "/webarenaIndigo/v1/dns/addCNAMERecord",
        cname_body.clone(),
        "D80022",
    )
    .await;
    assert_eq!(
        api.dns()
            .add_cname_record(cname_request())
            .await
            .unwrap()
            .success_code,
        "D80022"
    );
    mock.assert_async().await;
    let mock = mock_record(
        &mut server,
        "PUT",
        "/webarenaIndigo/v1/dns/updateCNAMERecord",
        cname_body,
        "D80023",
    )
    .await;
    assert_eq!(
        api.dns()
            .update_cname_record(cname_request())
            .await
            .unwrap()
            .success_code,
        "D80023"
    );
    mock.assert_async().await;

    let mx_body = json!({
        "zone_id": "102756",
        "domain_name": "example.com",
        "host_name": "www",
        "mail_server": "mail.example.com",
        "priority": "100",
        "ttl": "600"
    });
    let mx_request = || MxRecordRequest {
        zone_id: 102756,
        domain_name: "example.com".to_string(),
        host_name: "www".to_string(),
        mail_server: "mail.example.com".to_string(),
        priority: 100,
        ttl: 600,
    };
    let mock = mock_record(
        &mut server,
        "POST",
        "/webarenaIndigo/v1/dns/addMXRecord",
        mx_body.clone(),
        "D80010",
    )
    .await;
    assert_eq!(
        api.dns()
            .add_mx_record(mx_request())
            .await
            .unwrap()
            .success_code,
        "D80010"
    );
    mock.assert_async().await;
    let mock = mock_record(
        &mut server,
        "PUT",
        "/webarenaIndigo/v1/dns/updateMXRecord",
        mx_body,
        "D80011",
    )
    .await;
    assert_eq!(
        api.dns()
            .update_mx_record(mx_request())
            .await
            .unwrap()
            .success_code,
        "D80011"
    );
    mock.assert_async().await;

    let txt_body = json!({
        "zone_id": "102756",
        "domain_name": "example.com",
        "host_name": "www",
        "txt_value": "v=spf1 -all",
        "ttl": "600"
    });
    let txt_request = || TxtRecordRequest {
        zone_id: 102756,
        domain_name: "example.com".to_string(),
        host_name: "www".to_string(),
        txt_value: "v=spf1 -all".to_string(),
        ttl: 600,
    };
    let mock = mock_record(
        &mut server,
        "POST",
        "/webarenaIndigo/v1/dns/addTXTRecord",
        txt_body.clone(),
        "D80013",
    )
    .await;
    assert_eq!(
        api.dns()
            .add_txt_record(txt_request())
            .await
            .unwrap()
            .success_code,
        "D80013"
    );
    mock.assert_async().await;
    let mock = mock_record(
        &mut server,
        "PUT",
        "/webarenaIndigo/v1/dns/updateTXTRecord",
        txt_body,
        "D80014",
    )
    .await;
    assert_eq!(
        api.dns()
            .update_txt_record(txt_request())
            .await
            .unwrap()
            .success_code,
        "D80014"
    );
    mock.assert_async().await;

    let ns_body = json!({
        "zone_id": "102756",
        "domain_name": "example.com",
        "host_name": "www",
        "to_server": ["example1.com", "example2.com"],
        "ttl": "600"
    });
    let ns_request = || NsRecordRequest {
        zone_id: 102756,
        domain_name: "example.com".to_string(),
        host_name: "www".to_string(),
        to_server: vec!["example1.com".to_string(), "example2.com".to_string()],
        ttl: 600,
    };
    let mock = mock_record(
        &mut server,
        "POST",
        "/webarenaIndigo/v1/dns/addNSRecord",
        ns_body.clone(),
        "D80016",
    )
    .await;
    assert_eq!(
        api.dns()
            .add_ns_record(ns_request())
            .await
            .unwrap()
            .success_code,
        "D80016"
    );
    mock.assert_async().await;
    let mock = mock_record(
        &mut server,
        "PUT",
        "/webarenaIndigo/v1/dns/updateNSRecord",
        ns_body,
        "D80017",
    )
    .await;
    assert_eq!(
        api.dns()
            .update_ns_record(ns_request())
            .await
            .unwrap()
            .success_code,
        "D80017"
    );
    mock.assert_async().await;

    let srv_body = json!({
        "zone_id": "102756",
        "domain_name": "example.com",
        "host_name": "www",
        "priority": 10,
        "target": "app2.",
        "weight": 50,
        "port": 80,
        "ttl": "600"
    });
    let srv_request = || SrvRecordRequest {
        zone_id: 102756,
        domain_name: "example.com".to_string(),
        host_name: "www".to_string(),
        priority: 10,
        target: "app2.".to_string(),
        weight: 50,
        port: 80,
        ttl: 600,
    };
    let mock = mock_record(
        &mut server,
        "POST",
        "/webarenaIndigo/v1/dns/addSRVRecord",
        srv_body.clone(),
        "D80019",
    )
    .await;
    assert_eq!(
        api.dns()
            .add_srv_record(srv_request())
            .await
            .unwrap()
            .success_code,
        "D80019"
    );
    mock.assert_async().await;
    let mock = mock_record(
        &mut server,
        "PUT",
        "/webarenaIndigo/v1/dns/updateSRVRecord",
        srv_body,
        "D80020",
    )
    .await;
    assert_eq!(
        api.dns()
            .update_srv_record(srv_request())
            .await
            .unwrap()
            .success_code,
        "D80020"
    );
    mock.assert_async().await;

    let delete_body = json!({
        "zone_id": "102756",
        "domain_name": "example.com",
        "host_name": "www",
        "type": "A"
    });
    let mock = mock_record(
        &mut server,
        "DELETE",
        "/webarenaIndigo/v1/dns/deleteNSRecord",
        delete_body,
        "D80006",
    )
    .await;
    let res = api
        .dns()
        .destroy_record(DeleteRecordRequest {
            zone_id: 102756,
            domain_name: "example.com".to_string(),
            host_name: "www".to_string(),
            record_type: DnsRecordType::A,
        })
        .await
        .unwrap();
    assert_eq!(res.success_code, "D80006");
    mock.assert_async().await;
}

#[tokio::test]
async fn status_error() {
    let (mut server, api) = server_and_api().await;
    server
        .mock("GET", "/webarenaIndigo/v1/vm/sshkey")
        .with_status(401)
        .with_body(r#"{"message":"Unauthorized"}"#)
        .create_async()
        .await;

    let err = api.ssh().ssh_key_list().await.err().unwrap();
    match err {
        WebArenaIndigoApiError::Status(status, body) => {
            assert_eq!(status, 401);
            assert!(body.contains("Unauthorized"));
        }
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn json_parse_error() {
    let (mut server, api) = server_and_api().await;
    server
        .mock("GET", "/webarenaIndigo/v1/vm/sshkey")
        .with_body("not json")
        .create_async()
        .await;

    let err = api.ssh().ssh_key_list().await.err().unwrap();
    match err {
        WebArenaIndigoApiError::JsonParse(body) => assert_eq!(body, "not json"),
        other => panic!("unexpected error: {:?}", other),
    }
}

#[tokio::test]
async fn rate_limit_retry() {
    let (mut server, api) = server_and_api().await;
    let mock = server
        .mock("GET", "/webarenaIndigo/v1/vm/sshkey")
        .with_status(429)
        .with_body(r#"{"errorCode": "429", "errorMessage": "Too Many Request."}"#)
        .expect_at_least(3)
        .create_async()
        .await;

    let err = api.ssh().ssh_key_list().await.err().unwrap();
    match err {
        WebArenaIndigoApiError::Status(status, _) => assert_eq!(status, 429),
        other => panic!("unexpected error: {:?}", other),
    }
    // A 429 is retried twice, so three requests are sent in total.
    mock.assert_async().await;
}

#[tokio::test]
async fn connection_error() {
    // An unsupported URL scheme fails instantly without touching the network.
    let api = WebArenaIndigoApi::new("id", "secret")
        .with_base_url("unsupported://127.0.0.1")
        .with_access_token(TOKEN);

    let err = api.ssh().ssh_key_list().await.err().unwrap();
    assert!(matches!(err, WebArenaIndigoApiError::Connection(_)));
}
