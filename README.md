# web-arena-indigo

[![Crates.io](https://img.shields.io/crates/v/web-arena-indigo.svg)](https://crates.io/crates/web-arena-indigo)
[![docs.rs](https://img.shields.io/docsrs/web-arena-indigo)](https://docs.rs/web-arena-indigo)
[![CI](https://github.com/uiuifree/rust-web-arena-indigo/actions/workflows/ci.yml/badge.svg)](https://github.com/uiuifree/rust-web-arena-indigo/actions/workflows/ci.yml)

**Unofficial** async Rust client for the [WebARENA Indigo](https://web.arena.ne.jp/indigo/)
VPS API by NTTPC Communications. Covers every endpoint of the official REST API:
instances (VMs), SSH keys, API keys, firewalls, snapshots, and DNS.
This library is a community project and is not affiliated with or endorsed by
NTTPC Communications.

NTTPC の VPS サービス「WebARENA Indigo」の**非公式** Rust API クライアントです
（NTTPC コミュニケーションズとは無関係のコミュニティ製ライブラリです）。
公式 REST API の全エンドポイント（インスタンス・SSH キー・API キー・
ファイアウォール・スナップショット・DNS）に対応しています。

- Official API reference: <https://indigo.arena.ne.jp/userapi/>
- Crate documentation: <https://docs.rs/web-arena-indigo>
- Runnable examples: [`examples/`](examples/)

## Installation

```toml
[dependencies]
web-arena-indigo = "0.2"
```

MSRV: 1.86

## Quick start

```rust,no_run
use web_arena_indigo::{CreateInstanceRequest, WebArenaIndigoApi};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Client ID / secret issued from the Indigo control panel
    let mut api = WebArenaIndigoApi::new("CLIENT_ID", "CLIENT_SECRET");
    api.update_access_token().await?;

    // List instances
    for vm in api.instance().instance_list().await? {
        println!("{} {} {:?}", vm.instance_name, vm.status, vm.ipaddress);
    }

    // Create an instance
    api.instance()
        .create_instance(CreateInstanceRequest::Standard {
            ssh_key_id: 15532,
            region_id: 1,
            os_id: 13,
            instance_plan: 1,
            instance_name: "my-instance".to_string(),
        })
        .await?;

    // Add a DNS A record
    use web_arena_indigo::dns::ARecordRequest;
    api.dns()
        .add_a_record(ARecordRequest {
            zone_id: 102756,
            domain_name: "example.com".to_string(),
            host_name: "dev".to_string(),
            address: vec!["192.0.2.1".to_string()],
            ttl: 600,
        })
        .await?;

    Ok(())
}
```

## Authentication

`update_access_token` issues an OAuth access token from the client ID / secret and stores
it in the client. To reuse a token you obtained earlier, skip the call:

```rust,ignore
let api = WebArenaIndigoApi::new("CLIENT_ID", "CLIENT_SECRET").with_access_token("TOKEN");
```

`issued_at()` (epoch milliseconds) and `expires_in()` (seconds) expose the token metadata
for expiry handling.

## API coverage

All 51 methods of the official API are implemented. Response types are exported from the
module of each group (`web_arena_indigo::instance`, `web_arena_indigo::dns`, ...).

### Instance (VM) — `api.instance()`

| Rust method | Endpoint |
| --- | --- |
| `instance_type_list` | `GET /webarenaIndigo/v1/vm/instancetypes` |
| `region_list` | `GET /webarenaIndigo/v1/vm/getregion` |
| `os_list` | `GET /webarenaIndigo/v1/vm/oslist` |
| `instance_specification` | `GET /webarenaIndigo/v1/vm/getinstancespec` |
| `create_instance` | `POST /webarenaIndigo/v1/vm/createinstance` |
| `instance_list` | `GET /webarenaIndigo/v1/vm/getinstancelist` |
| `update_instance_status` | `POST /webarenaIndigo/v1/vm/instance/statusupdate` |

### SSH Key — `api.ssh()`

| Rust method | Endpoint |
| --- | --- |
| `ssh_key_list` | `GET /webarenaIndigo/v1/vm/sshkey` |
| `active_ssh_key_list` | `GET /webarenaIndigo/v1/vm/sshkey/active/status` |
| `create_ssh_key` | `POST /webarenaIndigo/v1/vm/sshkey` |
| `retrieve_ssh_key` | `GET /webarenaIndigo/v1/vm/sshkey/{id}` |
| `update_ssh_key` | `PUT /webarenaIndigo/v1/vm/sshkey/{id}` |
| `destroy_ssh_key` | `DELETE /webarenaIndigo/v1/vm/sshkey/{id}` |

### API Key — `api.api_key()`

| Rust method | Endpoint |
| --- | --- |
| `create_api_key` | `GET /webarenaIndigo/v1/auth/create/apikey` |
| `api_key_list` | `GET /webarenaIndigo/v1/auth/apikey` |
| `destroy_api_key` | `DELETE /webarenaIndigo/v1/auth/apikey/{id}` |

### Firewall — `api.firewall()`

| Rust method | Endpoint |
| --- | --- |
| `create_firewall` | `POST /webarenaIndigo/v1/nw/createfirewall` |
| `firewall_list` | `GET /webarenaIndigo/v1/nw/getfirewalllist` |
| `retrieve_firewall` | `GET /webarenaIndigo/v1/nw/gettemplate/{id}` |
| `update_firewall` | `PUT /webarenaIndigo/v1/nw/updatefirewall` |
| `assign_firewall` | `POST /webarenaIndigo/v1/nw/assign` |
| `destroy_firewall` | `DELETE /webarenaIndigo/v1/nw/deletefirewall/{id}` |

### Snapshot — `api.snapshot()`

| Rust method | Endpoint |
| --- | --- |
| `take_snapshot` | `POST /webarenaIndigo/v1/disk/takesnapshot` |
| `snapshot_list` | `GET /webarenaIndigo/v1/disk/snapshotlist/{instance_id}` |
| `retake_snapshot` | `POST /webarenaIndigo/v1/disk/retakesnapshot` |
| `restore_snapshot` | `POST /webarenaIndigo/v1/disk/restoresnapshot` |
| `destroy_snapshot` | `DELETE /webarenaIndigo/v1/disk/deletesnapshot/{id}` |

### DNS — `api.dns()`

| Rust method | Endpoint |
| --- | --- |
| `register_domain` | `POST /webarenaIndigo/v1/dns/registerdomain` |
| `domain_list` | `GET /webarenaIndigo/v1/dns/getdomainlist` |
| `retrieve_domain` | `GET /webarenaIndigo/v1/dns/getdomainrecord/{id}` |
| `cancel_domain` | `DELETE /webarenaIndigo/v1/dns/canceldomain/{id}` |
| `create_domain_verification_token` | `POST /webarenaIndigo/v1/dns/domainverificationtoken` |
| `cancel_domain_verification` | `DELETE /webarenaIndigo/v1/dns/canceldomainverification/{id}` |
| `verify_domain` | `GET /webarenaIndigo/v1/dns/domainverification/{id}` |
| `domain_records` | `GET /webarenaIndigo/v1/dns/editdomain/{id}` |
| `ttl_list` | `GET /webarenaIndigo/v1/dns/getttl` |
| `add_a_record` | `POST /webarenaIndigo/v1/dns/addARecord` |
| `update_a_record` | `PUT /webarenaIndigo/v1/dns/updateARecord` |
| `add_aaaa_record` | `POST /webarenaIndigo/v1/dns/addAAAARecord` |
| `update_aaaa_record` | `PUT /webarenaIndigo/v1/dns/updateAAAARecord` |
| `add_cname_record` | `POST /webarenaIndigo/v1/dns/addCNAMERecord` |
| `update_cname_record` | `PUT /webarenaIndigo/v1/dns/updateCNAMERecord` |
| `add_mx_record` | `POST /webarenaIndigo/v1/dns/addMXRecord` |
| `update_mx_record` | `PUT /webarenaIndigo/v1/dns/updateMXRecord` |
| `add_txt_record` | `POST /webarenaIndigo/v1/dns/addTXTRecord` |
| `update_txt_record` | `PUT /webarenaIndigo/v1/dns/updateTXTRecord` |
| `add_ns_record` | `POST /webarenaIndigo/v1/dns/addNSRecord` |
| `update_ns_record` | `PUT /webarenaIndigo/v1/dns/updateNSRecord` |
| `add_srv_record` | `POST /webarenaIndigo/v1/dns/addSRVRecord` |
| `update_srv_record` | `PUT /webarenaIndigo/v1/dns/updateSRVRecord` |
| `destroy_record` | `DELETE /webarenaIndigo/v1/dns/deleteNSRecord` |

## Error handling

Every method returns `Result<T, WebArenaIndigoApiError>`:

```rust,ignore
use web_arena_indigo::WebArenaIndigoApiError;

match api.instance().instance_list().await {
    Ok(instances) => println!("{} instances", instances.len()),
    Err(WebArenaIndigoApiError::Status(code, body)) => eprintln!("API error {code}: {body}"),
    Err(WebArenaIndigoApiError::JsonParse(body)) => eprintln!("unexpected response: {body}"),
    Err(WebArenaIndigoApiError::Connection(e)) => eprintln!("network error: {e}"),
}
```

## Implementation notes

- The API gateway rate-limits aggressively (spike arrest: 2 requests/second, burst 1).
  The client paces requests at one per 600 ms by default and retries twice on
  `429 Too Many Requests`; tune with `with_request_interval` (pass `Duration::ZERO`
  to disable, e.g. against a mock server). On top of the spike arrest there is an
  account-level quota (roughly 10 requests/minute was observed); for long call
  sequences raise the interval, e.g. `with_request_interval(Duration::from_secs(7))`.
- Right after `create_instance` the instance rejects operations for a while
  ("Instance ID is invalid"), and an assigned firewall template cannot be deleted
  while its instance is active (`F60009`).
- The API spells `sucessCode` (sic) in responses; the crate exposes it as `success_code`.
- Fields the API returns as either numbers or strings (e.g. `vps_kind`, snapshot
  `user_id` / `size`, `DNSZoneId`) are accepted in both forms.
- Numeric parameters the API expects as JSON strings (`zone_id`, `ttl`, `templateid`, ...)
  are exposed as `u32` and converted on serialization.

## Testing

```sh
# Offline tests (mock server, no credentials needed)
cargo test --test mock_api

# Tests against the real API (skipped unless C_KEY / C_SECRET_KEY are set)
cargo test --test live_api
```

## License

MIT
