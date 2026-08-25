mod config;
mod copier;
mod watcher;

use anyhow::Result;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    println!("========================================");
    println!(" Fingerprint Database Copy");
    println!("========================================");

    let config = Config::load("config.toml")?;

    println!("Database:");
    println!("  {}", config.database);

    println!();
    println!("Replicas:");

    for replica in &config.replicas {
        println!(
            "  {} -> {}:{}",
            replica.name,
            replica.address,
            replica.destination
        );
    }

    println!();

    watcher::run(config).await?;

    Ok(())
}