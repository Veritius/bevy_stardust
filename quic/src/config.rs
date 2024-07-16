use std::{fmt::Debug, sync::Arc};
use anyhow::bail;

/// A builder for an [`AppProtos`] set.
pub struct AppProtosBuilder {
    protos: Vec<AppProto>,
}

impl AppProtosBuilder {
    /// Create a new empty [`AppProtosBuilder`].
    pub fn new() -> Self {
        AppProtosBuilder {
            protos: Vec::default(),
        }
    }

    /// Try to create an [`AppProtos`] from the builder.
    /// Fails if no [`AppProto`] items have been added.
    pub fn build(self) -> anyhow::Result<AppProtos> {
        // The set must have at least one item
        if self.protos.len() == 0 {
            bail!("Length of AppProtos was zero");
        }

        // Create and return the new structure
        return Ok(AppProtos(AppProtosInner {
            inner: self.protos.into(),
        }));
    }
}

impl Default for AppProtosBuilder {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Extend<AppProto> for AppProtosBuilder {
    fn extend<T: IntoIterator<Item = AppProto>>(&mut self, iter: T) {
        self.protos.extend(iter.into_iter())
    }
}

/// A set of one or more [ALPN] application prototype codes.
/// 
/// Constructed using an [`AppProtosBuilder`] instance.
/// 
/// This is cheaply clonable and can be reused.
/// 
/// [ALPN]: https://www.rfc-editor.org/rfc/rfc7301.html
#[derive(Clone)]
pub struct AppProtos(AppProtosInner);

impl AsRef<[AppProto]> for AppProtos {
    fn as_ref(&self) -> &[AppProto] {
        self.0.inner.as_ref()
    }
}

impl Debug for AppProtos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.inner.fmt(f)
    }
}

#[derive(Clone)]
struct AppProtosInner {
    inner: Arc<[AppProto]>,
}

/// A single [ALPN] application prototype code, with UTF-8 encoding.
/// 
/// This is cheaply clonable and can be reused.
/// 
/// [ALPN]: https://www.rfc-editor.org/rfc/rfc7301.html
#[derive(Clone)]
pub struct AppProto(AppProtoInner);

impl AsRef<str> for AppProto {
    fn as_ref(&self) -> &str {
        &self.0.inner
    }
}

impl AsRef<[u8]> for AppProto {
    fn as_ref(&self) -> &[u8] {
        self.0.inner.as_bytes()
    }
}

impl From<Box<str>> for AppProto {
    fn from(value: Box<str>) -> Self {
        Self(AppProtoInner { inner: value.into() })
    }
}

impl From<Arc<str>> for AppProto {
    fn from(value: Arc<str>) -> Self {
        Self(AppProtoInner { inner: value.into() })
    }
}

impl From<String> for AppProto {
    fn from(value: String) -> Self {
        Self(AppProtoInner { inner: value.into() })
    }
}

impl Debug for AppProto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.inner.as_ref().fmt(f)
    }
}

#[derive(Clone)]
struct AppProtoInner {
    inner: Arc<str>,
}