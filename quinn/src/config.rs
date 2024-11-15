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

pub enum ServerAuthentication {
    Authenticated {
        cert_chain: FetchTask<Vec<CertificateDer<'static>>>,
        private_key: FetchTask<PrivateKeyDer<'static>>,
    },

    Disabled,
}

impl ServerAuthentication {
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

pub enum ServerVerification {
    Authenticated {
        root_certs: FetchTask<RootCertStore>,
    },
}

impl ServerVerification {
    pub fn from_files(
        root_certs: impl Into<PathBuf>,
    ) -> Self {
        return Self::Authenticated {
            root_certs: LoadFromFile::new(root_certs).into(),
        }
    }
}

pub enum ClientAuthentication {
    Disabled,
}

pub enum ClientVerification {
    Disabled,
}

pub struct LoadFromFile<T> {
    pub path: PathBuf,

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