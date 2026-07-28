use std::path::Path;
use std::sync::Arc;

const CERT_FILE: &str = "cert.der";
const KEY_FILE: &str = "key.der";

// Loads the server's certificate/key from disk, or generates a new self-signed pair onfirst run. `cert.der` (not `key.der`) is the file you copy to each client as`server_cert.der` so they can recognize this exact server.
pub fn load_or_create_identity() -> (rustls::Certificate, rustls::PrivateKey) {
    if Path::new(CERT_FILE).exists() && Path::new(KEY_FILE).exists() {
        let cert = std::fs::read(CERT_FILE).expect("failed to read cert.der");
        let key = std::fs::read(KEY_FILE).expect("failed to read key.der");
        (rustls::Certificate(cert), rustls::PrivateKey(key))
    } else {
        let generated = rcgen::generate_simple_self_signed(vec!["filesync".into()])
            .expect("failed to generate self-signed certificate");
        let cert_der = generated.serialize_der().expect("failed to serialize certificate");
        let key_der = generated.serialize_private_key_der();
        std::fs::write(CERT_FILE, &cert_der).expect("failed to write cert.der");
        std::fs::write(KEY_FILE, &key_der).expect("failed to write key.der");
        log::info!("generated a new self-signed certificate ({CERT_FILE}) - copy it to clients as server_cert.der");
        (rustls::Certificate(cert_der), rustls::PrivateKey(key_der))
    }
}

pub fn build_server_config(cert: rustls::Certificate, key: rustls::PrivateKey) -> Arc<rustls::ServerConfig> {
    let config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .expect("invalid certificate/key pair");
    Arc::new(config)
}
