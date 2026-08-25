use crate::config::Replica;
use anyhow::{anyhow, Context, Result};
use std::path::Path;
use tokio::process::Command;

pub async fn copy_to_replica(
    source: &str,
    replica: &Replica,
) -> Result<()> {
    println!(
        "[COPY] {} -> {}:{}",
        source,
        replica.address,
        replica.destination
    );

    let source = format!("{}/", source);

    let destination = format!(
        "{}:{}/",
        replica.address,
        replica.destination
    );

    let status = Command::new("rsync")
        .arg("-a")
        .arg("--delete")
        .arg("--whole-file")
        .arg("--human-readable")
        .arg(&source)
        .arg(&destination)
        .status()
        .await
        .context("failed to start rsync")?;

    if !status.success() {
        return Err(anyhow!(
            "rsync to {} failed with status {}",
            replica.name,
            status
        ));
    }

    println!("[OK] {} updated", replica.name);

    Ok(())
}

pub async fn copy_to_all(
    source: &str,
    replicas: &[Replica],
) {
    let source_path = Path::new(source);

    if !source_path.exists() {
        eprintln!(
            "[ERROR] Database directory does not exist: {}",
            source
        );
        return;
    }

    println!();
    println!("========================================");
    println!("Copying database to {} replicas", replicas.len());
    println!("========================================");

    // Copy to all Pis concurrently.
    let mut tasks = Vec::new();

    for replica in replicas {
        let source = source.to_owned();
        let replica = replica.clone();

        tasks.push(tokio::spawn(async move {
            if let Err(error) = copy_to_replica(&source, &replica).await {
                eprintln!(
                    "[ERROR] {}: {:#}",
                    replica.name,
                    error
                );
            }
        }));
    }

    for task in tasks {
        let _ = task.await;
    }

    println!("========================================");
    println!("Copy complete");
    println!("========================================");
    println!();
}