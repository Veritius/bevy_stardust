use std::{path::{Path, PathBuf}, sync::Arc, io::BufReader};
use rustls::{ServerConfig, cipher_suite::{TLS13_AES_256_GCM_SHA384, TLS13_AES_128_GCM_SHA256, TLS13_CHACHA20_POLY1305_SHA256}, RootCertStore, version::TLS13, Certificate, server::AllowAnyAnonymousOrAuthenticatedClient};
use crate::CommandLineArguments;

pub(super) fn server_config(args: &CommandLineArguments) -> Arc<ServerConfig> {
    // Process all certificates in cert file
    let cert_path = args.cert_path.parse::<PathBuf>().expect("Invalid certificate path");
    let roots = read_certificate(cert_path.as_path());
    let mut client_auth_roots = RootCertStore::empty();
    for root in roots { client_auth_roots.add(&root); }
    let client_auth = AllowAnyAnonymousOrAuthenticatedClient::new(client_auth_roots).boxed();


    let mut config = ServerConfig::builder()
        .with_cipher_suites(&[
            TLS13_AES_256_GCM_SHA384,
            TLS13_AES_128_GCM_SHA256,
            TLS13_CHACHA20_POLY1305_SHA256,
        ])
        .with_safe_default_kx_groups()
        .with_protocol_versions(&[&TLS13])
        .expect("Inconsistent cipher-suites/versions specified.")
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert_with_ocsp_and_sct(cert_chain, key_der, ocsp, scts)
        .expect("Bad certificates/private key");

    todo!()
}

fn read_certificate(path: &Path) -> Vec<Certificate> {
    let certfile = std::fs::File::open(path).expect(&format!("Could not open certificate file at {}", path.display()));
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| Certificate(v.clone()))
        .collect()
}