use std::{path::PathBuf, io::BufReader, net::SocketAddr};
use bevy::prelude::Resource;
use bevy_stardust_shared::rustls::{Certificate, PrivateKey};

/// Configures the TLS web server that clients use for key exchange.
#[derive(Resource)]
pub struct AuthenticationServerConfig {
    /// Address for the webserver to bind to.
    pub address: SocketAddr,
    /// Certificates for the server to send to clients.
    pub certificates: Vec<Certificate>,
    /// The private key of this server. Whatever you do, **keep this secret.**
    pub private_key: PrivateKey,
}

fn read_certificates(path: &str) -> Vec<Certificate> {
    let path = path.parse::<PathBuf>().expect("Could not parse path");
    let certfile = std::fs::File::open(&path).expect(&format!("Could not open certificate file at {}", path.display()));
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| Certificate(v.clone()))
        .collect()
}

fn read_private_key(path: &str) -> PrivateKey {
    let keyfile = std::fs::File::open(path).expect("Could not open given path");
    let mut reader = BufReader::new(keyfile);

    loop {
        use rustls_pemfile::Item::*;
        match rustls_pemfile::read_one(&mut reader).expect("Could not parse certificate file") {
            Some(RSAKey(key)) => return PrivateKey(key),
            Some(PKCS8Key(key)) => return PrivateKey(key),
            Some(ECKey(key)) => return PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!("No keys found in file {:?}", path);
}