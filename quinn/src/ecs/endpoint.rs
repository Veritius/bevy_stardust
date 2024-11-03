use std::sync::Arc;
use bevy_ecs::prelude::*;
use crate::{backend::endpoint::{EndpointKey, EndpointRef}, config::*, Connection, QuicManager};

/// A QUIC endpoint.
#[derive(Component)]
pub struct Endpoint {
    pub(crate) inner: EndpointRef,

    key: EndpointKey,
}

impl Endpoint {
    /// Creates a new [`Endpoint`] component.
    pub fn new(
        manager: &QuicManager,
        auth: ServerAuthentication,
        verify: ClientVerification,
    ) -> Endpoint {
        let (key, inner) = crate::backend::endpoint::create(
            manager.executor.executor_arc(),
            auth,
            verify
        );

        return Endpoint {
            inner,
            key,
        };
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