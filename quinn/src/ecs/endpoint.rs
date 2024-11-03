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
        todo!()
    }

    /// Creates a new [`Connection`] component.
    pub fn connect(
        &self,
        manager: &QuicManager,
        auth: ClientAuthentication,
        verify: ServerVerification,
    ) -> Connection {
        todo!()
    }
}