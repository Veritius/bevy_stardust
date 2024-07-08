use bevy::prelude::*;

/// A QUIC connection.
/// 
/// All connections 'belong' to an [`Endpoint`](crate::Endpoint), which they use for I/O.
#[derive(Component, Reflect)]
#[reflect(from_reflect=false, Component)]
pub struct Connection {
    #[reflect(ignore)]
    endpoint: Entity,

    #[cfg(feature="quiche")]
    #[reflect(ignore)]
    quiche: quiche::Connection,
}