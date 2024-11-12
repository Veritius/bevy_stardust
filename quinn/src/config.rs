use std::{future::Future, path::PathBuf, pin::Pin};
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

type FetchTask<V> = Pin<Box<dyn Fetch<V>>>;

pub enum ServerAuthentication {
    Authenticated {
        cert_chain: FetchTask<Vec<CertificateDer<'static>>>,
        private_key: FetchTask<PrivateKeyDer<'static>>,
    },

    Disabled,
}

pub enum ServerVerification {
    Authenticated {
        root_certs: FetchTask<RootCertStore>,
    },
}

pub enum ClientAuthentication {
    Disabled,
}

pub enum ClientVerification {
    Disabled,
}

pub struct LoadCertsFromFile(pub PathBuf);

impl Into<FetchTask<Vec<CertificateDer<'static>>>> for LoadCertsFromFile {
    fn into(self) -> FetchTask<Vec<CertificateDer<'static>>> {
        Box::pin(async move {
            // Read the entire path to a string
            // TODO: Async version of this ?
            let str = std::fs::read_to_string(self.0)?;

            todo!()
        })
    }
}