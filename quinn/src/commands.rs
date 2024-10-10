use std::{borrow::Cow, net::SocketAddr, sync::Arc};
use bevy_ecs::{prelude::*, system::{EntityCommand, EntityCommands}};
use quinn_proto::{ClientConfig, ServerConfig, EndpointConfig};
use crate::{connection::{ConnectionBundle, ConnectionInner}, endpoint::EndpointInner, Connection, Endpoint, QuicSocket};

/// Extension API to sugar using endpoint commands.
pub trait EndpointCommands {
    /// Makes the target entity an endpoint, sugaring [`MakeEndpoint`].
    /// 
    /// Fails if the entity does not exist, or is already an endpoint.
    fn make_endpoint(
        &mut self,
        config: MakeEndpoint,
    ) -> &mut Self;

    /// Opens a connection on an endpoint, sugaring [`OpenConnection`].
    /// 
    /// Fails if the entity does not exist, is not an endpoint, or is closing.
    fn connect(
        &mut self,
        config: OpenConnection,
    ) -> &mut Self;
}

impl EndpointCommands for EntityWorldMut<'_> {
    fn make_endpoint(
        &mut self,
        config: MakeEndpoint,
    ) -> &mut Self {
        let id = self.id();
        self.world_scope(|world| {
            config.apply(id, world);
        });

        return self;
    }

    fn connect(
        &mut self,
        config: OpenConnection,
    ) -> &mut Self {
        let id = self.id();
        self.world_scope(|world| {
            config.apply(id, world);
        });

        return self;
    }
}

impl EndpointCommands for EntityCommands<'_> {
    #[inline]
    fn make_endpoint(
        &mut self,
        config: MakeEndpoint,
    ) -> &mut Self {
        self.add(config);
        return self;
    }

    #[inline]
    fn connect(
        &mut self,
        config: OpenConnection,
    ) -> &mut Self {
        self.add(config);
        return self;
    }
}

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

            None => {
                #[cfg(feature="log")]
                bevy_log::warn!("Tried to make {id} an endpoint but it did not exist");

                return;
            },
        };

        // Check that the entity isn't already an endpoint
        if entity.contains::<Endpoint>() {
            #[cfg(feature="log")]
            bevy_log::warn!("Tried to make {id} an endpoint it was already one");

            return;
        }

        // Construct the endpoint component
        let endpoint = Endpoint::new(
            self.socket,
            self.config,
            self.server,
        );

        // Add the endpoint component
        entity.insert(endpoint);

        #[cfg(feature="log")]
        bevy_log::info!("Opened endpoint {}", entity.id());
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

            None => {
                #[cfg(feature="log")]
                bevy_log::warn!("Tried to access {endpoint_id} as an endpoint, but the entity did not exist");

                return;
            },
        };

        // Try to access the component
        let mut endpoint = match endpoint.get_mut::<Endpoint>() {
            Some(endpoint) => endpoint,

            None => {
                #[cfg(feature="log")]
                bevy_log::warn!("Tried to access {endpoint_id} as an endpoint, but it was not an endpoint");

                return;
            },
        };

        // Try to create a connection with the endpoint
        match unsafe { endpoint.init_remote_connection(
            connection_id,
            self.config,
            self.remote,
            &self.server_name,
        ) } {
            Ok((handle, connection)) => {
                // Connection state machine
                let statemachine = bevy_stardust_quic::Connection::new();

                // Spawn the new connection entity
                world.get_or_spawn(connection_id)
                    .unwrap() // Shouldn't happen
                    .insert(ConnectionBundle::new(unsafe { Connection(Box::new(ConnectionInner::new(
                        handle,
                        endpoint_id,
                        connection,
                        statemachine,
                    )))}));
                
                #[cfg(feature="log")]
                bevy_log::info!("Created new outgoing connection to {} on {endpoint_id} with id {connection_id}", self.remote);
            },

            Err(err) => {
                #[cfg(feature="log")]
                bevy_log::error!("Failed to create outgoing connection to {} on {endpoint_id}: {}", self.remote, err);

                return;
            },
        }
    }
}