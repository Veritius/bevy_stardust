use std::{borrow::Cow, net::SocketAddr, sync::Arc};
use bevy_ecs::{prelude::*, system::EntityCommand};
use quinn_proto::{ClientConfig, ServerConfig, EndpointConfig};
use crate::{connection::ConnectionInner, endpoint::EndpointInner, Connection, Endpoint, QuicSocket};

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

        // Check that the entity isn't already an endpoint
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

    /// The name of the server.
    pub server_name: Cow<'static, str>,
}

impl EntityCommand for OpenConnection {
    fn apply(
        self,
        endpoint_id: Entity,
        world: &mut World,
    ) {
        // Reserve one connection ID that we use if the connection succeeds
        let connection_id = world.entities().reserve_entity();

        // Try to get access to the entity
        let mut endpoint: EntityWorldMut<'_> = match world.get_entity_mut(endpoint_id) {
            Some(endpoint) => endpoint,

            #[cfg(feature="log")]
            None => {
                use bevy_log::prelude::*;

                warn!("Tried to access {endpoint_id} as an endpoint, but the entity did not exist");

                return;
            },

            #[cfg(not(feature="log"))]
            None => return, // Do nothing
        };

        // Try to access the component
        let mut endpoint = match endpoint.get_mut::<Endpoint>() {
            Some(endpoint) => endpoint,

            #[cfg(feature="log")]
            None => {
                use bevy_log::prelude::*;

                warn!("Tried to access {endpoint_id} as an endpoint, but it was not an endpoint");

                return;
            },

            #[cfg(not(feature="log"))]
            None => return, // Do nothing
        };

        // Try to create a connection with the endpoint
        match unsafe { endpoint.0.init_remote_connection(
            endpoint_id,
            self.config,
            self.remote,
            &self.server_name,
        ) } {
            Ok((handle, connection)) => {
                // Spawn the new connection entity
                world.get_or_spawn(connection_id)
                    .unwrap() // Shouldn't happen
                    .insert(unsafe { Connection(Box::new(ConnectionInner::new(
                        handle,
                        endpoint_id,
                        connection,
                    )))});
                
                #[cfg(feature="log")]
                {
                    use bevy_log::prelude::*;
    
                    error!("Created new outgoing connection to {} on {endpoint_id} with id {connection_id}",
                        self.remote);
                }
            },

            Err(err) => {
                #[cfg(feature="log")]
                {
                    use bevy_log::prelude::*;

                    error!("Failed to create outgoing connection to {} on {endpoint_id}: {}",
                        self.remote, err);
                }

                return;
            },
        }
    }
}