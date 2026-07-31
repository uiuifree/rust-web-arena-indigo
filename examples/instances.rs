//! Lists instance types, regions, OSes and your instances.
//!
//! Usage: `C_KEY=... C_SECRET_KEY=... cargo run --example instances`
//! (or put the variables in a `.env` file)

use web_arena_indigo::WebArenaIndigoApi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let mut api = WebArenaIndigoApi::new(std::env::var("C_KEY")?, std::env::var("C_SECRET_KEY")?);
    api.update_access_token().await?;

    for instance_type in api.instance().instance_type_list().await?.instance_types {
        println!(
            "instance type {}: {}",
            instance_type.id, instance_type.display_name
        );
        for region in api
            .instance()
            .region_list(instance_type.id)
            .await?
            .region_list
        {
            println!("  region {}: {}", region.id, region.name);
        }
        for category in api.instance().os_list(instance_type.id).await?.os_category {
            for os in category.os_lists {
                println!("  os {}: {}", os.id, os.view_name);
            }
        }
    }

    for vm in api.instance().instance_list().await? {
        println!("{} {} {:?}", vm.instance_name, vm.status, vm.ipaddress);
    }
    Ok(())
}
