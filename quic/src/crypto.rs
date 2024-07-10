use std::sync::Arc;

/// A private key used for encryption.
#[derive(Clone)]
pub struct PrivateKey(PrivateKeyInner);

impl PrivateKey {
    /// Create a `PrivateKey` from a PEM-encoded slice.
    pub fn from_pem(pem: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        return Ok(Self::from_boring_pkey(boring::pkey::PKey::private_key_from_pem(pem.as_ref())?));
    }

    /// Create a `PrivateKey` from a DER-encoded slice.
    pub fn from_der(der: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        return Ok(Self::from_boring_pkey(boring::pkey::PKey::private_key_from_der(der.as_ref())?));
    }

    /// Create a `PrivateKey` from a DER-encoded PKCS#8 key.
    pub fn from_pkcs8(pkcs8: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        return Ok(Self::from_boring_pkey(boring::pkey::PKey::private_key_from_pkcs8(pkcs8.as_ref())?));
    }

    #[cfg(feature="quiche")]
    fn from_boring_pkey(inner: boring::pkey::PKey<boring::pkey::Private>) -> Self {
        Self(PrivateKeyInner { inner: Arc::new(inner) })
    }
}

#[derive(Clone)]
struct PrivateKeyInner {
    #[cfg(feature="quiche")]
    inner: Arc<boring::pkey::PKey<boring::pkey::Private>>,
}

/// An X.509 certificate used for encryption.
#[derive(Clone)]
pub struct Certificate(CertificateInner);

impl Certificate {
    /// Create a `Certificate` from a PEM-encoded slice.
    pub fn from_pem(pem: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        return Ok(Self::from_boring_x509(boring::x509::X509::from_pem(pem.as_ref())?));
    }

    /// Create a `Certificate` from a DER-encoded slice.
    pub fn from_der(der: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        return Ok(Self::from_boring_x509(boring::x509::X509::from_der(der.as_ref())?));
    }

    #[cfg(feature="quiche")]
    fn from_boring_x509(inner: boring::x509::X509) -> Self {
        Self(CertificateInner { inner: Arc::new(inner) })
    }
}

#[derive(Clone)]
struct CertificateInner {
    #[cfg(feature="quiche")]
    inner: Arc<boring::x509::X509>,
}

/// A chain of [`Certificate`] objects.
#[derive(Clone)]
pub struct CertChain(CertChainInner);

impl CertChain {
    /// Create a `CertChain` from an iterator of certificates.
    pub fn from_iter<I: Iterator<Item = Certificate>>(iter: I) -> anyhow::Result<Self> {
        todo!()
    }
}

#[derive(Clone)]
struct CertChainInner {

}

/// A collection of trusted root certificates.
#[derive(Clone)]
pub struct RootCAs(RootCAsInner);

impl RootCAs {
    /// Create a `RootCAs` from an iterator of `CertChain` objects.
    pub fn from_iter<I: Iterator<Item = CertChain>>(iter: I) -> anyhow::Result<Self> {
        todo!()
    }
}

#[derive(Clone)]
struct RootCAsInner {

}