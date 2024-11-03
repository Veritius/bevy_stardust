use std::sync::Arc;
use bevy_ecs::prelude::*;
use crate::{backend::connection::ConnectionRef, config::*, Endpoint, QuicManager};

/// A QUIC connection.
#[derive(Component)]
pub struct Connection {
    inner: ConnectionRef,
}

impl Connection {
    /// Creates a new [`Connection`] through an [`Endpoint`].
    pub fn new(
        manager: &QuicManager,
        endpoint: &Endpoint,
        auth: ClientAuthentication,
        verify: ServerVerification,
        server_name: Arc<str>,
    ) -> Self {
        todo!()
    }
}