use crate::config::Config;
use crate::copier::copy_to_all;

use anyhow::Result;
use notify::{
    Config as NotifyConfig,
    Event,
    EventKind,
    RecommendedWatcher,
    RecursiveMode,
    Watcher,
};

use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

pub async fn run(config: Config) -> Result<()> {
    let (tx, rx) = channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(
        move |result| {
            if let Err(error) = tx.send(result) {
                eprintln!("Failed to send filesystem event: {error}");
            }
        },
        NotifyConfig::default(),
    )?;

    watcher.watch(
        Path::new(&config.database),
        RecursiveMode::Recursive,
    )?;

    println!(
        "Watching database: {}",
        config.database
    );

    println!(
        "Debounce: {} ms",
        config.debounce_ms
    );

    loop {
        let first_event = match rx.recv() {
            Ok(Ok(event)) => event,
            Ok(Err(error)) => {
                eprintln!("[WATCHER ERROR] {error}");
                continue;
            }
            Err(error) => {
                return Err(error.into());
            }
        };

        if !is_relevant_event(&first_event) {
            continue;
        }

        println!(
            "[CHANGE] {:?}",
            first_event.paths
        );

        // Wait for the burst of filesystem activity to finish.
        let mut last_change = Instant::now();

        loop {
            match rx.recv_timeout(Duration::from_millis(
                config.debounce_ms,
            )) {
                Ok(Ok(event)) => {
                    if is_relevant_event(&event) {
                        println!(
                            "[CHANGE] {:?}",
                            event.paths
                        );

                        last_change = Instant::now();
                    }
                }

                Ok(Err(error)) => {
                    eprintln!("[WATCHER ERROR] {error}");
                }

                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if last_change.elapsed()
                        >= Duration::from_millis(config.debounce_ms)
                    {
                        break;
                    }
                }

                Err(error) => {
                    return Err(error.into());
                }
            }
        }

        println!("[SYNC] Database changed");

        copy_to_all(
            &config.database,
            &config.replicas,
        )
        .await;
    }
}

fn is_relevant_event(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_)
            | EventKind::Modify(_)
            | EventKind::Remove(_)
    )
}