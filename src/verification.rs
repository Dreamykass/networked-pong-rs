use std::sync::Arc;

struct SkipCertificationVerification;

impl rustls::ServerCertVerifier for SkipCertificationVerification {
    fn verify_server_cert(
        &self,
        _: &rustls::RootCertStore,
        _: &[rustls::Certificate],
        _: webpki::DNSNameRef,
        _: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}

pub fn new_insecure_client_config() -> quinn::ClientConfig {
    let mut cfg = quinn::ClientConfigBuilder::default().build();

    // Get a mutable reference to the 'crypto' config in the 'client config'.
    let tls_cfg: &mut rustls::ClientConfig = std::sync::Arc::get_mut(&mut cfg.crypto).unwrap();

    // Change the certification verifier.
    // This is only available when compiled with the 'dangerous_configuration' feature.
    tls_cfg
        .dangerous()
        .set_certificate_verifier(Arc::new(SkipCertificationVerification));
    cfg
}

pub fn generate_self_signed_cert() -> anyhow::Result<(quinn::Certificate, quinn::PrivateKey)> {
    let rcgen_certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let serialized_key = rcgen_certificate.serialize_private_key_der();
    let serialized_certificate = rcgen_certificate.serialize_der().unwrap();

    // Write to files.
    // fs::write(CERT_PATH, &serialized_certificate).context("failed to write certificate")?;
    // fs::write(CERT_KEY, &serialized_key).context("failed to write private key")?;

    let cert = quinn::Certificate::from_der(&serialized_certificate)?;
    let key = quinn::PrivateKey::from_der(&serialized_key)?;
    Ok((cert, key))
}

// pub fn load_cert() -> anyhow::Result<(quinn::Certificate, quinn::PrivateKey)> {
//     let serialized_cert = fs::read(CERT_PATH).context("")?;
//     let serialized_key = fs::read(CERT_KEY).context("")?;
// }
