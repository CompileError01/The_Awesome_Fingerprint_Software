mod config;
mod tls;

use common::{read_message, Message};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;

const RECONNECT_DELAY: Duration = Duration::from_secs(3);
const PINNED_CERT_FILE: &str = "server_cert.der";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let cfg = config::load("config.toml");
    let sync_folder = PathBuf::from(&cfg.sync_folder);
    std::fs::create_dir_all(&sync_folder)?;
    let addr = format!("{}:{}", cfg.server_ip, cfg.server_port);

    let pinned_cert = std::fs::read(PINNED_CERT_FILE).unwrap_or_else(|e| {
        panic!("could not read {PINNED_CERT_FILE}: {e} - copy cert.der from the server here and rename it")
    });
    let connector = TlsConnector::from(Arc::new(tls::build_client_config(pinned_cert)));
    // Required by the TLS API but unused for verification here, since PinnedCertVerifier
    // checks the certificate bytes directly instead of matching a hostname.
    let server_name = rustls::ServerName::try_from("filesync-server").unwrap();

    // Outer loop: whenever the connection ends (server restart, network blip, first start),
    // wait a bit and try again. This is what makes the client resilient to drops.
    loop {
        match TcpStream::connect(&addr).await {
            Ok(tcp_stream) => match connector.connect(server_name.clone(), tcp_stream).await {
                Ok(tls_stream) => {
                    log::info!("connected to {addr} (TLS)");
                    if let Err(e) = run_session(tls_stream, &sync_folder).await {
                        log::warn!("connection lost: {e}");
                    }
                }
                Err(e) => log::warn!("TLS handshake with {addr} failed: {e}"),
            },
            Err(e) => log::warn!("could not connect to {addr}: {e}"),
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

// Reads messages until the connection breaks, applying each one to the local mirror.
async fn run_session(mut stream: TlsStream<TcpStream>, folder: &Path) -> std::io::Result<()> {
    loop {
        let msg = read_message(&mut stream).await?;
        apply(&msg, folder);
    }
}

fn apply(msg: &Message, folder: &Path) {
    match msg {
        Message::File { path, data } => {
            let target = folder.join(path);
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&target, data) {
                Ok(()) => log::info!("synced {path}"),
                Err(e) => log::warn!("failed to write {}: {e}", target.display()),
            }
        }
        Message::Delete { path } => {
            let target = folder.join(path);
            if target.is_file() && std::fs::remove_file(&target).is_ok() {
                log::info!("deleted {path}");
            }
        }
    }
}
