use std::{net::SocketAddr, sync::Arc};
use bevy_ecs::{prelude::*, system::EntityCommand};
use quinn_proto::{ClientConfig, ServerConfig, EndpointConfig};
use crate::{endpoint::EndpointInner, Endpoint, QuicSocket};

/// Creates a new QUIC endpoint with this entity.
pub struct MakeEndpoint {
    /// The UDP socket to use.
    pub socket: QuicSocket,

    /// The configuration of the endpoint.
    pub config: Arc<EndpointConfig>,

    /// The server configuration of the endpoint.
    pub server: Option<Arc<ServerConfig>>,
}

impl EntityCommand for MakeEndpoint {
    fn apply(
        self,
        id: Entity,
        world: &mut World,
    ) {
        // Try to get access to the entity
        let mut entity = match world.get_entity_mut(id) {
            Some(entity) => entity,

            #[cfg(feature="log")]
            None => {
                use bevy_log::prelude::*;

                warn!("Tried to make {id} an endpoint but it did not exist");

                return;
            },

            #[cfg(not(feature="log"))]
            None => return, // Do nothing
        };

        // Check that the entity isn't already an endpoing
        if entity.contains::<Endpoint>() {
            #[cfg(feature="log")]
            {
                use bevy_log::prelude::*;

                warn!("Tried to make {id} an endpoint it was already one");
            }

            return;
        }

        // Construct the endpoint component
        let endpoint = Endpoint(Box::new(EndpointInner::new(
            self.socket,
            self.config,
            self.server
        )));

        // Add the endpoint component
        entity.insert(endpoint);
    }
}

/// Creates a new QUIC connection based on an endpoint.
pub struct OpenConnection {
    /// The address of the remote server to connect to.
    pub remote: SocketAddr,

    /// The configuration of the client.
    pub config: ClientConfig,
}

impl EntityCommand for OpenConnection {
    fn apply(
        self,
        id: Entity,
        world: &mut World,
    ) {
        todo!()
    }
}