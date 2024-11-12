use std::future::Future;
use rustls::{pki_types::{CertificateDer, PrivateKeyDer}, RootCertStore};

/// An operation to asynchronously retrieve a value from disk or wherever else it may be stored.
/// Often used when loading configuration or cryptography data.
pub trait Fetch<V>
where
    Self: Send + 'static,
    Self: Future<Output = std::io::Result<V>>,
{}

impl<T, V> Fetch<V> for T
where
    T: Send + 'static,
    T: Future<Output = std::io::Result<V>>,
{}

pub enum ServerAuthentication {
    Authenticated {
        cert_chain: Box<dyn Fetch<Vec<CertificateDer<'static>>>>,
        private_key: Box<dyn Fetch<PrivateKeyDer<'static>>>,
    },

    Disabled,
}

pub enum ServerVerification {
    Authenticated {
        root_certs: Box<dyn Fetch<RootCertStore>>,
    }
}

pub enum ClientAuthentication {
    Disabled,
}

pub enum ClientVerification {
    Disabled,
}