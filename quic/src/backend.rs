use std::any::Any;
use bevy::reflect::TypePath;
use crate::{connection::ConnectionState, endpoint::EndpointState};

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