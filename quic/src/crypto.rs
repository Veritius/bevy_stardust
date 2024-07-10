use std::sync::Arc;

/// A private key used for encryption.
#[derive(Clone)]
pub struct PrivateKey {
    inner: Arc<PrivateKeyInner>,
}

impl PrivateKey {
    /// Create a `PrivateKey` from a PEM-encoded slice.
    pub fn from_pem(pem: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        todo!()
    }

    /// Create a `PrivateKey` from a DER-encoded slice.
    pub fn from_der(der: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        todo!()
    }

    /// Create a `PrivateKey` from a DER-encoded PKCS#8 key.
    pub fn from_pkcs8(pkcs8: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        todo!()
    }
}

struct PrivateKeyInner {

}

/// An X.509 certificate used for encryption.
#[derive(Clone)]
pub struct Certificate {
    inner: Arc<CertificateInner>,
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

struct CertificateInner {

}

/// A chain of [`Certificate`] objects.
#[derive(Clone)]
pub struct CertChain {
    inner: Arc<CertChainInner>,
}

impl CertChain {
    /// Create a `CertChain` from an iterator of certificates.
    pub fn from_iter<I: Iterator<Item = Certificate>>(iter: I) -> anyhow::Result<Self> {
        todo!()
    }
}

struct CertChainInner {

}

/// A collection of trusted root certificates.
#[derive(Clone)]
pub struct RootCAs {
    inner: Arc<RootCAsInner>,
}

impl RootCAs {
    /// Create a `RootCAs` from an iterator of `CertChain` objects.
    pub fn from_iter<I: Iterator<Item = CertChain>>(iter: I) -> anyhow::Result<Self> {
        todo!()
    }
}

struct RootCAsInner {

}