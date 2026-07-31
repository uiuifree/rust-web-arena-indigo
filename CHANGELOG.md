# Changelog

## 0.2.1 (unreleased)

### Added
- API Key, Firewall, Snapshot and DNS APIs (all endpoints documented at
  https://indigo.arena.ne.jp/userapi/).
- `CreateInstanceRequest` enum covering all documented boot methods:
  `Standard`, `Windows`, `Import` and `FromSnapshot`.
- `WebArenaIndigoApi::with_base_url` / `with_access_token`, and
  `issued_at()` / `expires_in()` getters for token-expiry handling.
- Built-in request pacing (600 ms interval, configurable via
  `with_request_interval`) and automatic retry on `429 Too Many Requests`,
  matching the API gateway's rate limit of 2 requests/second with burst 1.
- Offline test suite (`tests/mock_api.rs`) covering every API method against a
  local mock server, plus CI (fmt / clippy / test / doc).

### Fixed
- `issuedAt` (epoch milliseconds) was parsed into `u32` and always overflowed
  to 0; token metadata is now `u64`.
- Response struct fields were private and unreadable from outside the crate;
  all response fields are now `pub`.
- Removed a stray debug `println!` from `update_instance_status`.
- Fields the API returns as either numbers or strings (`vps_kind`,
  snapshot `user_id` / `size`, `DNSZoneId`) are now accepted in both forms.

### Changed (breaking)
- `WebArenaIndigoApiError` now implements `std::error::Error` and `Display`
  instead of an inherent `to_string`.
- `RequestCreateInstance` struct replaced by the `CreateInstanceRequest` enum.
- Renames for consistency: `get_instance_list` → `instance_list`,
  `Vms` → `Instance`, `GetRegionListResponse` → `RegionListResponse`,
  `GetOsListResponse` → `OsListResponse`,
  `GetInstanceSpecification` → `InstanceSpecificationResponse`,
  `SpecList` → `InstanceSpec`, `SshKeyStatus::ACTIVE/DEACTIVE` →
  `Active`/`Deactive` (wire format unchanged), `sucess_code` fields →
  `success_code`.
- `instance()` / `ssh()` now take `&self` instead of `&mut self`.
- A single shared `reqwest::Client` is reused across requests;
  reqwest 0.11 → 0.12; `serde_derive` dependency removed.
- Edition 2024, MSRV 1.86 (driven by the reqwest dependency tree).

## 0.1.0

- Initial release: Access Token, Instance and SSH Key APIs.
