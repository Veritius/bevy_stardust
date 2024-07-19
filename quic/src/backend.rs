use std::any::Any;

use bevy::reflect::TypePath;
use crate::{connection::ConnectionBackend, endpoint::EndpointBackend};

/// An implementation of the QUIC protocol.
pub trait Backend
where 
    Self: Send + Sync,
    Self: Any + TypePath,
{
    /// Endpoint state.
    type EndpointState: EndpointBackend;

    /// Connection state.
    type ConnectionState: ConnectionBackend;
}
