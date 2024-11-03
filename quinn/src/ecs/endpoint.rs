use bevy_ecs::prelude::*;
use crate::{backend::endpoint::EndpointRef, config::ServerAuthentication, QuicManager};

/// A QUIC endpoint.
#[derive(Component)]
pub struct Endpoint {
    inner: EndpointRef,
}

impl Endpoint {
    /// Creates a new [`Endpoint`] component.
    pub fn new(
        manager: &mut QuicManager,
        auth: ServerAuthentication,
    ) -> Endpoint {
        todo!()
    }
}