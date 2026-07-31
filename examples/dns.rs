//! Lists DNS domains and TTL presets, and adds an A record when
//! `ZONE_ID` / `DOMAIN_NAME` are set.
//!
//! Usage: `C_KEY=... C_SECRET_KEY=... cargo run --example dns`

use web_arena_indigo::WebArenaIndigoApi;
use web_arena_indigo::dns::ARecordRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let mut api = WebArenaIndigoApi::new(std::env::var("C_KEY")?, std::env::var("C_SECRET_KEY")?);
    api.update_access_token().await?;

    for domain in api.dns().domain_list().await? {
        println!(
            "{} (zone_id: {}, status: {})",
            domain.zone, domain.zone_id, domain.verify_status
        );
    }
    for ttl in api.dns().ttl_list().await? {
        println!("ttl {}: {}s", ttl.name, ttl.value);
    }

    if let (Ok(zone_id), Ok(domain_name)) = (std::env::var("ZONE_ID"), std::env::var("DOMAIN_NAME"))
    {
        let res = api
            .dns()
            .add_a_record(ARecordRequest {
                zone_id: zone_id.parse()?,
                domain_name,
                host_name: "dev".to_string(),
                address: vec!["192.0.2.1".to_string()],
                ttl: 3600,
            })
            .await?;
        println!("{}", res.message);
    }
    Ok(())
}
