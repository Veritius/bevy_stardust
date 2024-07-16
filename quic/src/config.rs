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

    /// Adds a single [`AppProto`] to the set.
    pub fn push(&mut self, proto: AppProto) {
        self.protos.push(proto);
    }

    /// Try to create an [`AppProtos`] from the builder.
    /// Fails if no [`AppProto`] items have been added.
    pub fn build(self) -> anyhow::Result<AppProtos> {
        // The set must have at least one item
        if self.protos.len() == 0 {
            bail!("Length of AppProtos was zero");
        }

        // Create and return the new structure
        return Ok(AppProtos(AppProtosInner::Owned(self.protos.into())));
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

impl AppProtos {
    /// Create an `AppProtos` set from a `'static` slice of `AppProto` types.
    /// Usable in const contexts.
    pub const fn from_static_slice(value: &'static [AppProto]) -> Self {
        Self(AppProtosInner::Static(value))
    }

    /// Return a type that can be dereferenced into `&[&[u8]]`.
    /// 
    /// Since Arc<T> is a pointer to T, we can't return a slice of them.
    /// What we can do is collect the Ts into a slice of pointers and return that.
    /// This requires an allocation but is used very infrequently so it's not that bad.
    pub(crate) fn collect<'a>(&'a self) -> Box<[&'a [u8]]> {
        self.as_ref()
            .iter()
            .map(|v| v.as_ref())
            .collect()
    }
}

impl AsRef<[AppProto]> for AppProtos {
    fn as_ref(&self) -> &[AppProto] {
        match &self.0 {
            AppProtosInner::Owned(arc) => &*arc,
            AppProtosInner::Static(str) => str,
        }
    }
}

impl From<&'static [AppProto]> for AppProtos {
    fn from(value: &'static [AppProto]) -> Self {
        Self::from_static_slice(value)
    }
}

impl Debug for AppProtos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

#[derive(Clone)]
enum AppProtosInner {
    Owned(Arc<[AppProto]>),
    Static(&'static [AppProto]),
}

/// A single [ALPN] application prototype code, with UTF-8 encoding.
/// 
/// This is cheaply clonable and can be reused.
/// 
/// [ALPN]: https://www.rfc-editor.org/rfc/rfc7301.html
#[derive(Clone)]
pub struct AppProto(AppProtoInner);

impl AppProto {
    /// Create an `AppProto` from a `&'static str`.
    /// Usable in const contexts.
    pub const fn from_static_str(str: &'static str) -> Self {
        Self(AppProtoInner::Static(str))
    }
}

impl AsRef<str> for AppProto {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl AsRef<[u8]> for AppProto {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_bytes()
    }
}

impl From<&'static str> for AppProto {
    fn from(value: &'static str) -> Self {
        Self::from_static_str(value)
    }
}

impl From<Box<str>> for AppProto {
    fn from(value: Box<str>) -> Self {
        Self(AppProtoInner::Owned(value.into()))
    }
}

impl From<Arc<str>> for AppProto {
    fn from(value: Arc<str>) -> Self {
        Self(AppProtoInner::Owned(value.into()))
    }
}

impl From<String> for AppProto {
    fn from(value: String) -> Self {
        Self(AppProtoInner::Owned(value.into()))
    }
}

impl Debug for AppProto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[derive(Clone)]
enum AppProtoInner {
    Owned(Arc<str>),
    Static(&'static str),
}

impl AsRef<str> for AppProtoInner {
    fn as_ref(&self) -> &str {
        match self {
            AppProtoInner::Owned(arc) => &*arc,
            AppProtoInner::Static(str) => str,
        }
    }
}