use std::sync::Arc;

/// An X.509 certificate used for encryption.
#[derive(Clone)]
pub struct Certificate {
    inner: Arc<[u8]>,
}

impl Certificate {
    /// Create a `Certificate` from a PEM-encoded slice.
    pub fn from_pem(pem: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        todo!()
    }

    /// Create a `Certificate` from a DER-encoded slice.
    pub fn from_der(der: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        todo!()
    }
}

/// A chain of [`Certificate`] objects.
#[derive(Clone)]
pub struct CertChain {
    inner: Arc<[Certificate]>,
}

impl CertChain {
    /// Create a `CertChain` from an iterator of certificates.
    pub fn from_iter<I: Iterator<Item = Certificate>>(iter: I) -> anyhow::Result<Self> {
        todo!()
    }
}

/// A collection of trusted root certificates.
#[derive(Clone)]
pub struct RootCAs {
    inner: Arc<[Certificate]>,
}

impl RootCAs {
    /// Create a `RootCAs` from an iterator of certificates.
    pub fn from_iter<I: Iterator<Item = Certificate>>(iter: I) -> anyhow::Result<Self> {
        todo!()
    }
}