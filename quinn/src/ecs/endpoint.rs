use std::sync::Arc;
use bevy_ecs::prelude::*;
use crate::{backend::endpoint::EndpointRef, config::*, Connection, QuicManager};

/// A QUIC endpoint.
#[derive(Component)]
pub struct Endpoint {
    inner: EndpointRef,
}

impl Endpoint {
    /// Creates a new [`Endpoint`] component.
    pub fn new(
        manager: &QuicManager,
        auth: ServerAuthentication,
        verify: ClientVerification,
    ) -> Endpoint {
        Endpoint {
            inner: crate::backend::endpoint::create(
                manager.executor.executor_arc(),
                auth,
                verify
            ),
        }
    }

    /// Creates a new [`Connection`] component using this [`Endpoint`].
    pub fn connect(
        &self,
        manager: &QuicManager,
        auth: ClientAuthentication,
        verify: ServerVerification,
        server_name: Arc<str>,
    ) -> Connection {
        Connection::new(
            manager,
            self,
            auth,
            verify,
            server_name
        )
    }
}