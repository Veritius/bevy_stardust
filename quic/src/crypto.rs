use std::sync::Arc;

/// A private key used for encryption.
#[derive(Clone)]
pub struct PrivateKey(Arc<PrivateKeyInner>);

impl PrivateKey {
    /// Create a `PrivateKey` from a PEM-encoded slice.
    pub fn from_pem(pem: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        Ok(Self::from_boring_pkey(boring::pkey::PKey::private_key_from_pem(pem.as_ref())?))
    }

    /// Create a `PrivateKey` from a DER-encoded slice.
    pub fn from_der(der: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        Ok(Self::from_boring_pkey(boring::pkey::PKey::private_key_from_der(der.as_ref())?))
    }

    /// Create a `PrivateKey` from a DER-encoded PKCS#8 key.
    pub fn from_pkcs8(pkcs8: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        Ok(Self::from_boring_pkey(boring::pkey::PKey::private_key_from_pkcs8(pkcs8.as_ref())?))
    }

    #[cfg(feature="quiche")]
    fn from_boring_pkey(inner: boring::pkey::PKey<boring::pkey::Private>) -> Self {
        Self(Arc::new(PrivateKeyInner { inner }))
    }
}

struct PrivateKeyInner {
    #[cfg(feature="quiche")]
    inner: boring::pkey::PKey<boring::pkey::Private>,
}

/// An X.509 certificate used for encryption.
#[derive(Clone)]
pub struct Certificate(Arc<CertificateInner>);

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
        Self(Arc::new(CertificateInner { inner }))
    }
}

struct CertificateInner {
    #[cfg(feature="quiche")]
    inner: boring::x509::X509,
}

/// A chain of [`Certificate`] objects.
#[derive(Clone)]
pub struct CertChain(Arc<CertChainInner>);

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
pub struct RootCAs(Arc<RootCAsInner>);

impl RootCAs {
    /// Create a `RootCAs` from an iterator of `CertChain` objects.
    pub fn from_iter<I: Iterator<Item = CertChain>>(iter: I) -> anyhow::Result<Self> {
        todo!()
    }
}

struct RootCAsInner {

}