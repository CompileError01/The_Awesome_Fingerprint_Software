use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::{Certificate, ClientConfig, Error, ServerName};
use std::sync::Arc;
use std::time::SystemTime;

struct PinnedCertVerifier {
    expected_der: Vec<u8>,
}

impl ServerCertVerifier for PinnedCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        if end_entity.0 == self.expected_der {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(Error::General(
                "server presented a certificate that doesn't match server_cert.der".into(),
            ))
        }
    }
}

// Builds a client TLS config that only ever accepts the one certificate in `pinned_cert_der`.
pub fn build_client_config(pinned_cert_der: Vec<u8>) -> ClientConfig {
    ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(PinnedCertVerifier { expected_der: pinned_cert_der }))
        .with_no_client_auth()
}
