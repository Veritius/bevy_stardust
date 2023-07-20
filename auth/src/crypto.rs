use std::{path::Path, fs, io::BufReader};
use rustls::{Certificate, PrivateKey};

pub fn load_certificates(path: &Path) -> Vec<Certificate> {
    let certfile = fs::File::open(path).expect("Couldn't open given certficate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap().iter().map(|v| Certificate(v.clone())).collect()
}

pub fn load_private_key(path: &Path) -> PrivateKey {
    let keyfile = fs::File::open(path).expect("Couldn't open private key file");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).expect("Private key file could not be parsed") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return PrivateKey(key),
            Some(rustls_pemfile::Item::ECKey(key)) => return PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!("No keys found in private key file");
}