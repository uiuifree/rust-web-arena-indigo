//! Tests against the real WebARENA Indigo API.
//! They run only when `C_KEY` / `C_SECRET_KEY` are set (directly or via `.env`),
//! and are skipped otherwise so that CI can run without credentials.

use web_arena_indigo::dns::{ARecordRequest, DeleteRecordRequest, DnsRecordType, DnsVerifyType};
use web_arena_indigo::firewall::{CreateFirewallRequest, FirewallRule, UpdateFirewallRequest};
use web_arena_indigo::{
    CreateInstanceRequest, InstanceStatus, SshKeyStatus, UpdateSshKeyRequest, WebArenaIndigoApi,
    WebArenaIndigoApiError,
};

const TEST_PUBLIC_KEY: &str = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCTOBwEzlxQhE++5aYEGKpl6Ye8hRD9hjHj/owiu4VQXA2Ew4PLI9/1sbR/75sJ3Ku7x5+LD8QFU2xEDxAPeDI9+4v8vKswc1OofhAucy+GzLE8HzLHxv/0xfGHpMePYETLmHLr+pLhOCpFam1D58AatqMBJzYdXoagHYQHQ3yj4Iv1EWlDUe8SOC69/l5bGdevRkapFBlaQPgL7c/x3BSSV/bquy3fKk4w/o5fDDorOUtnCVt/fB7nciGhBez0K9ce1YMFkF0K/dBrhEFMNsJvElG45GVjc99QGcSn3oEAwzqT/4qzEoY+5OuBXIb7a13ozmdK0ilPWkdu28U4DnCCGw7x4iroEmc+KsBbIvSZOBDVfb7biAOJyXuAz6JPVVCmD6xOA/XlJIaecV+wBWvC+LZ/TO4tKCo/A4PKc7nHmclSH3T+acUu8WUuUmnSjQmYIOukFq9PwamqJkReGgNMrHpyBJFSX1hHYVC7mQdo3dhbpYY1+re8UVIccD75Okk= uiuifree@uiuifree-ubuntu";

fn credentials() -> Option<(String, String)> {
    dotenv::dotenv().ok();
    let key = std::env::var("C_KEY").ok()?;
    let secret = std::env::var("C_SECRET_KEY").ok()?;
    Some((key, secret))
}

async fn authorized_api() -> Option<WebArenaIndigoApi> {
    let Some((key, secret)) = credentials() else {
        eprintln!("skip: C_KEY / C_SECRET_KEY are not set");
        return None;
    };
    let mut api = WebArenaIndigoApi::new(key, secret);
    let res = api.update_access_token().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    assert!(api.issued_at() > 0);
    assert!(api.expires_in() > 0);
    Some(api)
}

#[tokio::test]
async fn instance_read_apis() {
    let Some(api) = authorized_api().await else {
        return;
    };

    let res = api.instance().instance_type_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let instance_type_id = res
        .unwrap()
        .instance_types
        .first()
        .map(|t| t.id)
        .unwrap_or(1);

    let res = api.instance().region_list(instance_type_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let res = api.instance().os_list(instance_type_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let res = api
        .instance()
        .instance_specification(instance_type_id)
        .await;
    assert!(res.is_ok(), "{}", res.err().unwrap());

    let res = api.instance().instance_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    for row in res.unwrap() {
        println!("{} {} {:?}", row.instance_name, row.status, row.ipaddress);
    }
}

/// Read-only calls for the API groups added in 0.2.0, to verify the response
/// types against the real API.
#[tokio::test]
async fn read_only_apis() {
    let Some(api) = authorized_api().await else {
        return;
    };

    let res = api.api_key().api_key_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let res = api.firewall().firewall_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let res = api.dns().domain_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let res = api.dns().ttl_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
}

/// Creates, updates and deletes a firewall template. The template is never
/// assigned to an instance, so running instances are not affected. Free of
/// charge and self-cleaning.
#[tokio::test]
async fn firewall_lifecycle() {
    let Some(api) = authorized_api().await else {
        return;
    };

    let rule = |rule_type: &str, port: &str| FirewallRule {
        rule_type: rule_type.to_string(),
        protocol: "TCP".to_string(),
        port: port.to_string(),
        source: "0.0.0.0".to_string(),
    };

    let res = api
        .firewall()
        .create_firewall(CreateFirewallRequest {
            name: "livetest".to_string(),
            inbound: vec![rule("HTTP", "80")],
            outbound: vec![rule("HTTPS", "443")],
            instances: vec![],
        })
        .await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let firewall_id = res.unwrap().firewall_id;

    // Run through deletion even if a step fails, so no template is left behind.
    let retrieve = api.firewall().retrieve_firewall(firewall_id).await;
    let update = api
        .firewall()
        .update_firewall(UpdateFirewallRequest {
            firewall_id,
            name: "livetest2".to_string(),
            inbound: vec![rule("SSH", "22")],
            outbound: vec![rule("HTTPS", "443")],
            instances: vec![],
        })
        .await;
    let destroy = api.firewall().destroy_firewall(firewall_id).await;

    assert!(retrieve.is_ok(), "{}", retrieve.err().unwrap());
    assert!(update.is_ok(), "{}", update.err().unwrap());
    assert!(destroy.is_ok(), "{}", destroy.err().unwrap());
}

/// Registers a DNS zone, issues a verification token, and cancels everything.
/// The zone stays unverified and is never served. Free of charge and
/// self-cleaning.
#[tokio::test]
async fn dns_domain_lifecycle() {
    let Some(api) = authorized_api().await else {
        return;
    };
    const TEST_DOMAIN: &str = "rust-web-arena-indigo-livetest.com";

    // Pick up a leftover zone if a previous run failed halfway.
    let existing = api.dns().domain_list().await.ok().and_then(|list| {
        list.into_iter()
            .find(|domain| domain.zone == TEST_DOMAIN)
            .map(|domain| domain.id)
    });
    let domain_id = match existing {
        Some(id) => id,
        None => {
            let res = api.dns().register_domain(TEST_DOMAIN).await;
            assert!(res.is_ok(), "{}", res.err().unwrap());
            res.unwrap().zone_id
        }
    };

    // Run through cancellation even if a step fails, so no zone is left behind.
    let retrieve = api.dns().retrieve_domain(domain_id).await;
    let token = api
        .dns()
        .create_domain_verification_token(domain_id, DnsVerifyType::Ns)
        .await;
    let cancel_verification = api.dns().cancel_domain_verification(domain_id).await;
    let cancel = api.dns().cancel_domain(domain_id).await;

    assert!(retrieve.is_ok(), "{}", retrieve.err().unwrap());
    assert!(token.is_ok(), "{}", token.err().unwrap());
    println!("verification token: {}", token.unwrap());
    assert!(
        cancel_verification.is_ok(),
        "{}",
        cancel_verification.err().unwrap()
    );
    assert!(cancel.is_ok(), "{}", cancel.err().unwrap());
}

/// Creates an additional API key and deletes it again. Only the id whose key
/// matches the newly issued one is deleted; the key currently in use (`C_KEY`)
/// is never touched, and is re-verified at the end.
#[tokio::test]
async fn api_key_lifecycle() {
    let Some(api) = authorized_api().await else {
        return;
    };
    let current_key = std::env::var("C_KEY").unwrap_or_default();

    let res = api.api_key().create_api_key().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let new_key = res.unwrap().api_key;
    assert_ne!(new_key, current_key);

    let res = api.api_key().api_key_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let new_id = res
        .unwrap()
        .api_keys
        .iter()
        .find(|k| k.api_key == new_key && k.api_key != current_key)
        .map(|k| k.id);
    let Some(new_id) = new_id else {
        panic!("created API key not found in the list; not deleting anything");
    };

    let res = api.api_key().destroy_api_key(new_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());

    // The original key must still be able to issue tokens.
    assert!(authorized_api().await.is_some());
}

/// Assigns a firewall template (SSH/HTTP/HTTPS inbound) to an instance,
/// preferring a stopped one, then deletes the template to detach it.
#[tokio::test]
async fn firewall_assign() {
    let Some(api) = authorized_api().await else {
        return;
    };

    let res = api.instance().instance_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let instances = res.unwrap();
    let Some(vm) = instances
        .iter()
        .find(|vm| vm.status.to_lowercase().contains("stop"))
        .or_else(|| instances.first())
    else {
        eprintln!("skip: no instance available");
        return;
    };
    println!("assign target: {} ({})", vm.instance_name, vm.status);

    let rule = |rule_type: &str, port: &str| FirewallRule {
        rule_type: rule_type.to_string(),
        protocol: "TCP".to_string(),
        port: port.to_string(),
        source: "0.0.0.0".to_string(),
    };
    let res = api
        .firewall()
        .create_firewall(CreateFirewallRequest {
            name: "livetest-assign".to_string(),
            inbound: vec![rule("SSH", "22"), rule("HTTP", "80"), rule("HTTPS", "443")],
            outbound: vec![rule("HTTP", "80"), rule("HTTPS", "443")],
            instances: vec![],
        })
        .await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let firewall_id = res.unwrap().firewall_id;

    let assign = api.firewall().assign_firewall(firewall_id, vm.id).await;
    let destroy = api.firewall().destroy_firewall(firewall_id).await;

    assert!(assign.is_ok(), "{}", assign.err().unwrap());
    assert!(
        destroy.is_ok(),
        "template {} is still assigned; remove it in the control panel: {}",
        firewall_id,
        destroy.err().unwrap()
    );
}

/// Adds, updates and deletes an A record on a verified zone, using a dedicated
/// host name so existing records are untouched. Skipped when no verified zone
/// exists.
#[tokio::test]
async fn dns_record_lifecycle() {
    let Some(api) = authorized_api().await else {
        return;
    };
    const TEST_HOST: &str = "rust-webarena-livetest";

    let res = api.dns().domain_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let Some(domain) = res.unwrap().into_iter().find(|d| d.zone_id != 0) else {
        eprintln!("skip: no verified DNS zone");
        return;
    };
    println!("record target zone: {} ({})", domain.zone, domain.zone_id);

    let record = |address: &str| ARecordRequest {
        zone_id: domain.zone_id,
        domain_name: domain.zone.clone(),
        host_name: TEST_HOST.to_string(),
        address: vec![address.to_string()],
        ttl: 300,
    };
    let add = api.dns().add_a_record(record("192.0.2.1")).await;
    let update = api.dns().update_a_record(record("192.0.2.2")).await;
    let destroy = api
        .dns()
        .destroy_record(DeleteRecordRequest {
            zone_id: domain.zone_id,
            domain_name: domain.zone.clone(),
            host_name: TEST_HOST.to_string(),
            record_type: DnsRecordType::A,
        })
        .await;

    assert!(add.is_ok(), "{}", add.err().unwrap());
    assert!(update.is_ok(), "{}", update.err().unwrap());
    assert!(destroy.is_ok(), "{}", destroy.err().unwrap());
}

/// Takes a snapshot into an empty slot only (existing snapshots are never
/// overwritten), waits for completion, then deletes it. Skipped when no
/// instance or no free slot is available.
#[tokio::test]
async fn snapshot_lifecycle() {
    let Some(api) = authorized_api().await else {
        return;
    };

    let res = api.instance().instance_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let Some(vm) = res.unwrap().into_iter().next() else {
        eprintln!("skip: no instance available");
        return;
    };

    let res = api.snapshot().snapshot_list(vm.id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let used: Vec<u32> = res
        .unwrap()
        .iter()
        .filter(|s| s.deleted == 0)
        .map(|s| s.slot_number)
        .collect();
    let Some(slot) = [0u32, 1].into_iter().find(|s| !used.contains(s)) else {
        eprintln!("skip: no free snapshot slot on {}", vm.instance_name);
        return;
    };
    println!("snapshot target: {} slot {}", vm.instance_name, slot);

    let res = api.snapshot().take_snapshot("livetest", vm.id, slot).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    assert_eq!(res.unwrap().status, 0);

    // Wait until the snapshot reaches "created", then delete it.
    let mut snapshot_id = None;
    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let list = api.snapshot().snapshot_list(vm.id).await;
        let Ok(list) = list else { continue };
        if let Some(snap) = list
            .iter()
            .find(|s| s.slot_number == slot && s.deleted == 0)
        {
            println!("snapshot {}: {}", snap.id, snap.status);
            if snap.status == "created" {
                snapshot_id = Some(snap.id);
                break;
            }
        }
    }
    let Some(snapshot_id) = snapshot_id else {
        panic!("snapshot did not reach 'created' in time; delete it manually from slot {slot}");
    };

    let res = api.snapshot().destroy_snapshot(snapshot_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    assert_eq!(res.unwrap().status, 0);
}

/// Creates a real instance on the smallest plan, verifies the create response,
/// assigns a firewall template while it exists, then destroys everything.
/// This test is billed (hourly rate of the smallest plan, a few yen at most),
/// so it only runs when opted into explicitly:
/// `cargo test --test live_api instance_create_lifecycle -- --ignored`
#[tokio::test]
#[ignore = "creates a billable instance (smallest plan, destroyed immediately)"]
async fn instance_create_lifecycle() {
    let Some((key, secret)) = credentials() else {
        eprintln!("skip: C_KEY / C_SECRET_KEY are not set");
        return;
    };
    // The account quota is roughly 10 requests/minute; this long test paces
    // itself well below that so it never trips the quota mid-flight.
    let mut api = WebArenaIndigoApi::new(key, secret)
        .with_request_interval(std::time::Duration::from_secs(7));
    let res = api.update_access_token().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let api = api;
    const INSTANCE_NAME: &str = "livetest-instance";

    // Pick up a leftover instance if a previous run failed to destroy it.
    let leftover = api.instance().instance_list().await.ok().and_then(|list| {
        list.into_iter()
            .find(|vm| vm.instance_name == INSTANCE_NAME)
            .map(|vm| vm.id)
    });
    if let Some(id) = leftover {
        let res = api
            .instance()
            .update_instance_status(id, InstanceStatus::Destroy)
            .await;
        assert!(
            res.is_ok(),
            "leftover instance {} could not be destroyed — remove it in the control panel: {}",
            id,
            res.err().unwrap()
        );
        println!("leftover instance {} destroyed; rerun the test", id);
        return;
    }

    // Resolve type, region, OS (newest Ubuntu) and the smallest plan.
    let res = api.instance().instance_type_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let type_id = res.unwrap().instance_types.first().expect("no type").id;

    let res = api.instance().region_list(type_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let region_id = res.unwrap().region_list.first().expect("no region").id;

    let res = api.instance().os_list(type_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let os_id = res
        .unwrap()
        .os_category
        .iter()
        .find(|c| c.name == "Ubuntu")
        .and_then(|c| c.os_lists.last())
        .expect("no Ubuntu OS")
        .id;

    let res = api.instance().instance_specification(type_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let specs = res.unwrap();
    let spec = specs
        .spec_list
        .iter()
        .min_by_key(|s| s.id)
        .expect("no plan");
    println!(
        "smallest plan: {} {} ({})",
        spec.id, spec.name, spec.description
    );

    // Reuse a leftover key from a failed run instead of creating a duplicate.
    let leftover_key = api.ssh().ssh_key_list().await.ok().and_then(|list| {
        list.ssh_keys
            .iter()
            .find(|k| k.name == "livetest-instance-key")
            .map(|k| k.id)
    });
    let ssh_key_id = match leftover_key {
        Some(id) => id,
        None => {
            let res = api
                .ssh()
                .create_ssh_key("livetest-instance-key", TEST_PUBLIC_KEY)
                .await;
            assert!(res.is_ok(), "{}", res.err().unwrap());
            res.unwrap().ssh_key.id
        }
    };

    // Retry the billable create through residual quota rejections.
    let mut created = None;
    for attempt in 0..3 {
        tokio::time::sleep(std::time::Duration::from_secs(if attempt == 0 {
            15
        } else {
            75
        }))
        .await;
        let res = api
            .instance()
            .create_instance(CreateInstanceRequest::Standard {
                ssh_key_id,
                region_id,
                os_id,
                instance_plan: spec.id,
                instance_name: INSTANCE_NAME.to_string(),
            })
            .await;
        let quota_hit = matches!(&res, Err(WebArenaIndigoApiError::Status(429, _)));
        created = Some(res);
        if !quota_hit {
            break;
        }
        println!("create attempt {} hit the quota; waiting", attempt);
    }
    let created = created.expect("create was not attempted");
    let instance_id = created.as_ref().ok().map(|c| c.vms.id);

    // While the instance exists, verify firewall assignment against it.
    // Right after creation the API rejects operations on the instance
    // ("Instance ID is invalid"), so let provisioning settle first.
    let mut assign = None;
    let mut template_id = None;
    if let Some(id) = instance_id {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        let rule = FirewallRule {
            rule_type: "SSH".to_string(),
            protocol: "TCP".to_string(),
            port: "22".to_string(),
            source: "0.0.0.0".to_string(),
        };
        // Reuse a leftover template from a failed run instead of duplicating it.
        let leftover_template = api.firewall().firewall_list().await.ok().and_then(|list| {
            list.iter()
                .find(|f| f.name == "livetest-assign")
                .map(|f| f.id)
        });
        let res = match leftover_template {
            Some(id) => Ok(id),
            None => api
                .firewall()
                .create_firewall(CreateFirewallRequest {
                    name: "livetest-assign".to_string(),
                    inbound: vec![rule.clone()],
                    outbound: vec![rule],
                    instances: vec![],
                })
                .await
                .map(|f| f.firewall_id),
        };
        match res {
            Ok(template) => {
                template_id = Some(template);
                for attempt in 0..4 {
                    let res = api.firewall().assign_firewall(template, id).await;
                    let done = res.is_ok();
                    assign = Some(res);
                    if done {
                        break;
                    }
                    println!("assign attempt {} failed; retrying", attempt);
                    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
                }
            }
            Err(e) => assign = Some(Err(e)),
        }
    }

    // Destroy the instance, retrying while it is still provisioning.
    let mut destroy = None;
    if let Some(id) = instance_id {
        for attempt in 0..12 {
            let res = api
                .instance()
                .update_instance_status(id, InstanceStatus::Destroy)
                .await;
            let done = res.is_ok();
            destroy = Some(res);
            if done {
                break;
            }
            println!("destroy attempt {} failed; retrying", attempt);
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        }
    }

    // A template cannot be deleted while it is assigned to an active instance
    // (F60009), so clean it up only after the instance is destroyed.
    let mut firewall_cleanup = None;
    if let Some(template) = template_id {
        for attempt in 0..4 {
            let res = api.firewall().destroy_firewall(template).await;
            let done = res.is_ok();
            firewall_cleanup = Some(res);
            if done {
                break;
            }
            println!("firewall cleanup attempt {} failed; retrying", attempt);
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
        }
    }
    let key_destroy = api.ssh().destroy_ssh_key(ssh_key_id).await;

    // All cleanup has been attempted; now judge the results.
    assert!(created.is_ok(), "{}", created.err().unwrap());
    let created = created.unwrap();
    println!(
        "created instance {} (plan: {}, status: {})",
        created.vms.id, created.vms.plan, created.vms.status
    );
    let destroy = destroy.expect("destroy was not attempted");
    assert!(
        destroy.is_ok(),
        "instance {} may still exist and accrue charges — destroy it in the control panel now: {}",
        created.vms.id,
        destroy.err().unwrap()
    );
    let assign = assign.expect("assign was not attempted");
    assert!(assign.is_ok(), "{}", assign.err().unwrap());
    let firewall_cleanup = firewall_cleanup.expect("firewall cleanup was not attempted");
    assert!(
        firewall_cleanup.is_ok(),
        "{}",
        firewall_cleanup.err().unwrap()
    );
    assert!(key_destroy.is_ok(), "{}", key_destroy.err().unwrap());
}

#[tokio::test]
async fn ssh_key_lifecycle() {
    let Some(api) = authorized_api().await else {
        return;
    };

    let res = api.ssh().ssh_key_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let res = api.ssh().active_ssh_key_list().await;
    assert!(res.is_ok(), "{}", res.err().unwrap());

    let res = api.ssh().create_ssh_key("test", TEST_PUBLIC_KEY).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
    let ssh_key_id = res.unwrap().ssh_key.id;

    let res = api.ssh().retrieve_ssh_key(ssh_key_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());

    let res = api
        .ssh()
        .update_ssh_key(
            ssh_key_id,
            UpdateSshKeyRequest {
                ssh_name: "test2".to_string(),
                ssh_key: TEST_PUBLIC_KEY.to_string(),
                ssh_key_status: SshKeyStatus::Active,
            },
        )
        .await;
    assert!(res.is_ok(), "{}", res.err().unwrap());

    let res = api.ssh().destroy_ssh_key(ssh_key_id).await;
    assert!(res.is_ok(), "{}", res.err().unwrap());
}
