use std::{future::Future, marker::PhantomData, path::PathBuf, pin::Pin};
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

/// An owned [`Future`] to fetch a value.
pub struct FetchTask<V> {
    task: Pin<Box<dyn Fetch<V>>>,
}

impl<V> FetchTask<V> {
    /// Pins a future and creates a [`FetchTask`].
    pub fn pin<T>(task: T) -> FetchTask<V>
    where
        T: Fetch<V>,
    {
        FetchTask {
            task: Box::pin(task)
        }
    }

    /// Wraps a `Pin<Box<dyn Fetch<V>>>` into a [`FetchTask`].
    pub fn from_box(task: Pin<Box<dyn Fetch<V>>>) -> FetchTask<V> {
        FetchTask { task }
    }
}

/// Configuration for authenticating an endpoint as a server.
pub enum ServerAuthentication {
    /// Full TLS authentication.
    Authenticated {
        /// The certificate chain to use for authentication.
        cert_chain: FetchTask<Vec<CertificateDer<'static>>>,
        /// The private key to use for authentication.
        private_key: FetchTask<PrivateKeyDer<'static>>,
    },

    /// Disable all authentication.
    Disabled,
}

impl ServerAuthentication {
    /// Convenience method for loading from common cryptographic files.
    pub fn from_files(
        cert_files: impl Into<PathBuf>,
        key_file: impl Into<PathBuf>,
    ) -> Self {
        return Self::Authenticated {
            cert_chain: LoadFromFile::new(cert_files).into(),
            private_key: LoadFromFile::new(key_file).into(),
        };
    }
}

/// Configuration for cryptographically verifying servers.
pub enum ServerVerification {
    /// Require server authentication.
    Authenticated {
        /// Root certificates to verify certificates against.
        root_certs: FetchTask<RootCertStore>,
    },
}

impl ServerVerification {
    /// Convenience method from loading from common cryptographic files.
    pub fn from_files(
        root_certs: impl Into<PathBuf>,
    ) -> Self {
        return Self::Authenticated {
            root_certs: LoadFromFile::new(root_certs).into(),
        }
    }
}

/// Configuration for authenticated outgoing connections.
pub enum ClientAuthentication {
    /// Don't authenticate outgoing connections.
    Disabled,
}

/// Configuration for cryptographically verifying incoming connections.
pub enum ClientVerification {
    /// Don't authenticate incoming connections.
    Disabled,
}

/// Convenience type for loading certain types from disk.
pub struct LoadFromFile<T> {
    path: PathBuf,
    _p: PhantomData<T>,
}

impl<T> LoadFromFile<T> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            _p: PhantomData,
        }
    }
}

impl Into<FetchTask<Vec<CertificateDer<'static>>>> for LoadFromFile<Vec<CertificateDer<'static>>> {
    fn into(self) -> FetchTask<Vec<CertificateDer<'static>>> {
        FetchTask::pin(async move {
            // Read the entire path to a string
            // TODO: Async version of this ?
            let str = std::fs::read_to_string(self.path)?;

            todo!()
        })
    }
}

impl Into<FetchTask<PrivateKeyDer<'static>>> for LoadFromFile<PrivateKeyDer<'static>> {
    fn into(self) -> FetchTask<PrivateKeyDer<'static>> {
        FetchTask::pin(async move {
            // Read the entire path to a string
            // TODO: Async version of this ?
            let str = std::fs::read_to_string(self.path)?;

            todo!()
        })
    }
}

impl Into<FetchTask<RootCertStore>> for LoadFromFile<RootCertStore> {
    fn into(self) -> FetchTask<RootCertStore> {
        FetchTask::pin(async move {
            // Read the entire path to a string
            // TODO: Async version of this ?
            let str = std::fs::read_to_string(self.path)?;

            todo!()
        })
    }
}