use std::sync::Arc;
use bevy_ecs::prelude::*;
use crate::{backend::connection::ConnectionRef, config::*, Endpoint, QuicManager};

/// A QUIC connection.
#[derive(Component)]
pub struct Connection {
    pub(crate) inner: ConnectionRef,
}

impl Connection {
    /// Creates a new [`Connection`] through an [`Endpoint`].
    pub fn new(
        manager: &QuicManager,
        endpoint: &Endpoint,
        auth: ClientAuthentication,
        verify: ServerVerification,
        server_name: Arc<str>,
    ) -> Connection {
        let inner = crate::backend::connection::create(
            manager.executor.executor_arc(),
            endpoint.inner.clone(),
            auth,
            verify,
            server_name,
        );

        return Connection {
            inner,
        };
    }
}