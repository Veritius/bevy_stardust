use bevy::prelude::*;

/// A QUIC connection.
/// 
/// All connections 'belong' to an [`Endpoint`](crate::Endpoint), which they use for I/O.
#[derive(Component)]
pub struct Connection {
    endpoint: Entity,

    #[cfg(feature="quiche")]
    quiche: quiche::Connection,
}