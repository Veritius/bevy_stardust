//! The QUIC abstraction API surface.

mod connection;
mod datagrams;
mod endpoint;
mod streams;

use std::{any::Any, sync::Arc};
use bevy::{prelude::Resource, reflect::TypePath};

pub use connection::*;
pub use datagrams::*;
pub use endpoint::*;
pub use streams::*;

/// An abstraction over the QUIC protocol, used by [`QuicBackendPlugin`](crate::plugin::QuicBackendPlugin).
pub trait QuicBackend
where 
    Self: Send + Sync,
    Self: Any + TypePath,
{
    /// Endpoint state.
    type EndpointState: EndpointState<Backend = Self>;

    /// Connection state.
    type ConnectionState: ConnectionState<Backend = Self>;
}

#[derive(Resource)]
pub(crate) struct BackendInstance<Backend: QuicBackend>(Arc<Backend>);

impl<Backend: QuicBackend> From<Backend> for BackendInstance<Backend> {
    fn from(value: Backend) -> Self {
        Self(Arc::new(value))
    }
}

impl<Backend: QuicBackend> AsRef<Backend> for BackendInstance<Backend> {
    fn as_ref(&self) -> &Backend {
        &self.0
    }
}

impl<Backend: QuicBackend> Clone for BackendInstance<Backend> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}