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
        Self(PrivateKeyInner { inner })
    }

    #[cfg(feature="quiche")]
    pub(crate) fn as_boring_pkey_ref(&self) -> &boring::pkey::PKeyRef<boring::pkey::Private> {
        self.0.inner.as_ref()
    }
}

#[cfg(all(feature="reveal", feature="quiche"))]
impl From<boring::pkey::PKey<boring::pkey::Private>> for PrivateKey {
    #[inline]
    fn from(value: boring::pkey::PKey<boring::pkey::Private>) -> Self {
        Self::from_boring_pkey(value)
    }
}

#[cfg(all(feature="reveal", feature="quiche"))]
impl AsRef<boring::pkey::PKeyRef<boring::pkey::Private>> for PrivateKey {
    fn as_ref(&self) -> &boring::pkey::PKeyRef<boring::pkey::Private> {
        self.0.inner.as_ref()
    }
}

#[derive(Clone)]
struct PrivateKeyInner {
    #[cfg(feature="quiche")]
    inner: boring::pkey::PKey<boring::pkey::Private>,
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

    #[cfg(feature="quiche")]
    pub(crate) fn as_boring_x509(&self) -> boring::x509::X509 {
        (*self.0.inner).clone()
    }

    #[cfg(feature="quiche")]
    pub(crate) fn as_boring_x509_ref(&self) -> &boring::x509::X509Ref {
        self.0.inner.as_ref()
    }
}

#[cfg(all(feature="reveal", feature="quiche"))]
impl From<boring::x509::X509> for Certificate {
    #[inline]
    fn from(value: boring::x509::X509) -> Self {
        Self::from_boring_x509(value)
    }
}

#[cfg(all(feature="reveal", feature="quiche"))]
impl AsRef<boring::x509::X509Ref> for Certificate {
    #[inline]
    fn as_ref(&self) -> &boring::x509::X509Ref {
        self.0.inner.as_ref()
    }
}

#[derive(Clone)]
struct CertificateInner {
    #[cfg(feature="quiche")]
    inner: Arc<boring::x509::X509>,
}

/// A complete chain of certificates, from the issuer to the end entity.
/// 
/// Contains at least one certificate, as zero certificates is not a chain.
#[derive(Clone)]
pub struct CertChain(CertChainInner);

impl CertChain {
    /// Create a `CertChain` from an iterator of certificates.
    pub fn from_iter<I: IntoIterator<Item = Certificate>>(iter: I) -> anyhow::Result<Self> {
        let iter = iter.into_iter();
        let certs = iter.collect::<Vec<_>>();
        if certs.len() == 0 { anyhow::bail!("Iterator must have at least 1 certificate") }

        todo!("Verify cert chain")
    }

    /// Decodes and verifies a `CertChain` from PEM.
    pub fn from_pem(pem: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        #[cfg(feature="quiche")]
        let inner = {
            let stack = boring::x509::X509::stack_from_pem(pem.as_ref())?;
            todo!("Verify PEM chain")
        };

        // Return the certificate chain
        return Ok(Self(CertChainInner { inner }));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = Certificate> + '_ {
        self.0.inner.iter().cloned()
    }
}

#[derive(Clone)]
struct CertChainInner {
    inner: Arc<[Certificate]>,
}

/// A collection of trusted root certificates.
#[derive(Clone)]
pub struct TrustAnchors(TrustAnchorsInner);

impl TrustAnchors {
    /// Create a `TrustAnchors` store from an iterator of certificates.
    pub fn from_iter<I: IntoIterator<Item = Certificate>>(iter: I) -> anyhow::Result<Self> {
        return Ok(Self(TrustAnchorsInner { inner: iter.into_iter().collect() }));
    }

    #[cfg(feature="quiche")]
    pub(crate) fn into_boring_x509_store(self) -> anyhow::Result<boring::x509::store::X509Store> {
        use boring::x509::store::X509StoreBuilder;
        let mut builder = X509StoreBuilder::new()?;
        let iter = self.0.inner.iter().map(|v| (*v.0.inner).clone());
        for cert in iter { builder.add_cert(cert)?; }
        Ok(builder.build())
    }
}

#[derive(Clone)]
struct TrustAnchorsInner {
    inner: Arc<[Certificate]>,
}

/// TLS credentials used to authenticate this peer to incoming connections.
#[derive(Clone)]
pub struct Credentials {
    pub(crate) certificates: CertChain,
    pub(crate) private_key: PrivateKey,
}

impl Credentials {
    /// Creates a new `Credentials` from component parts.
    pub fn new(
        certificates: CertChain,
        private_key: PrivateKey,
    ) -> Self {
        Self { certificates, private_key }
    }
}