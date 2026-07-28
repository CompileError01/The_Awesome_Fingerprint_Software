mod config;
mod tls;

use common::{send_message, Message};
use notify::{RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let cfg = config::load("config.toml");
    let sync_folder = PathBuf::from(&cfg.sync_folder);
    std::fs::create_dir_all(&sync_folder)?;
    let sync_folder = sync_folder.canonicalize()?;

    let (cert, key) = tls::load_or_create_identity();
    let acceptor = TlsAcceptor::from(tls::build_server_config(cert, key));

    // Every filesystem change is broadcast once here, and each connected client
    // gets its own subscription so a slow client can't block the others.
    let (tx, _rx) = broadcast::channel::<Message>(1024);
    spawn_watcher(sync_folder.clone(), tx.clone());

    let addr = format!("{}:{}", cfg.ip, cfg.port);
    let listener = TcpListener::bind(&addr).await?;
    log::info!("server listening on {addr} (TLS), sharing '{}'", sync_folder.display());

    loop {
        let (socket, peer) = listener.accept().await?;
        let rx = tx.subscribe();
        let folder = sync_folder.clone();
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            // TCP is accepted first, then upgraded to TLS - this handshake is what actually negotiates the encryption before any file data is sent.
            match acceptor.accept(socket).await {
                Ok(tls_socket) => {
                    log::info!("client connected: {peer}");
                    if let Err(e) = handle_client(tls_socket, folder, rx).await {
                        log::warn!("client {peer} disconnected: {e}");
                    }
                }
                Err(e) => log::warn!("TLS handshake with {peer} failed: {e}"),
            }
        });
    }
}

// Sends the full current folder to a freshly connected client, then streams live changes.
async fn handle_client(
    mut socket: TlsStream<TcpStream>,
    folder: PathBuf,
    mut rx: broadcast::Receiver<Message>,
) -> std::io::Result<()> {
    send_initial_snapshot(&mut socket, &folder).await?;

    loop {
        match rx.recv().await {
            Ok(msg) => send_message(&mut socket, &msg).await?,
            // A slow client fell behind the broadcast buffer; skip ahead instead of stalling everyone.
            Err(broadcast::error::RecvError::Lagged(n)) => {
                log::warn!("a client lagged behind by {n} messages");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
    Ok(())
}

async fn send_initial_snapshot(socket: &mut TlsStream<TcpStream>, folder: &Path) -> std::io::Result<()> {
    for path in walk_files(folder) {
        if let Some(rel) = common::to_relative(folder, &path) {
            if let Ok(data) = std::fs::read(&path) {
                send_message(socket, &Message::File { path: rel, data }).await?;
            }
        }
    }
    Ok(())
}

// Recursively lists every file (not directory) under `dir`.
fn walk_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk_files(&path));
            } else {
                out.push(path);
            }
        }
    }
    out
}

// Runs `notify` on its own OS thread (it's a sync API) and turns raw events into Messages.
fn spawn_watcher(folder: PathBuf, tx: broadcast::Sender<Message>) {
    std::thread::spawn(move || {
        let (evt_tx, evt_rx) = std_mpsc::channel();
        let mut watcher = notify::recommended_watcher(evt_tx).expect("failed to create watcher");
        watcher
            .watch(&folder, RecursiveMode::Recursive)
            .expect("failed to watch sync folder");

        for res in evt_rx {
            let Ok(event) = res else { continue };
            for path in event.paths {
                let Some(rel) = common::to_relative(&folder, &path) else { continue };
                match event.kind {
                    notify::EventKind::Remove(_) => {
                        let _ = tx.send(Message::Delete { path: rel });
                    }
                    notify::EventKind::Create(_) | notify::EventKind::Modify(_) => {
                        // Directory events land here too; only actual files have bytes to send.
                        if path.is_file() {
                            if let Ok(data) = std::fs::read(&path) {
                                let _ = tx.send(Message::File { path: rel, data });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}
