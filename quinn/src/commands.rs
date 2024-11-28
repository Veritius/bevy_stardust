use std::{net::{SocketAddr, UdpSocket}, sync::Arc};
use bevy_ecs::{prelude::*, system::{EntityCommand, EntityCommands}};

/// Extension API to sugar using endpoint commands.
pub trait EndpointCommands {
    /// Makes the target entity an endpoint, sugaring [`MakeEndpoint`].
    /// 
    /// Fails if the entity does not exist, or is already an endpoint.
    fn make_endpoint(
        &mut self,
        config: MakeEndpoint,
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
}

/// Extension API to sugar using connection commands.
pub trait ConnectionCommands {
    /// Makes the target entity a connection, sugaring [`OpenConnection`].
    /// 
    /// Fails if the entity does not exist, or is already a connection.
    fn open_connection(
        &mut self,
        config: OpenConnection,
    ) -> &mut Self;
}

impl ConnectionCommands for EntityWorldMut<'_> {
    fn open_connection(
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

impl ConnectionCommands for EntityCommands<'_> {
    #[inline]
    fn open_connection(
        &mut self,
        config: OpenConnection,
    ) -> &mut Self {
        self.add(config);
        return self;
    }
}

/// Opens a new endpoint with the target entity.
pub struct MakeEndpoint(MakeEndpointInner);

impl MakeEndpoint {
    pub fn advanced(
        socket: UdpSocket,
        config: Arc<quinn_proto::EndpointConfig>,
        server: Option<Arc<quinn_proto::ServerConfig>>,
    ) -> MakeEndpoint {
        let inner = MakeEndpointInner::Preconfigured {
            socket,
            config,
            server,
        };

        return MakeEndpoint(inner);
    }
}

pub(crate) enum MakeEndpointInner {
    Preconfigured {
        socket: UdpSocket,
        config: Arc<quinn_proto::EndpointConfig>,
        server: Option<Arc<quinn_proto::ServerConfig>>,
    },
}

impl EntityCommand for MakeEndpoint {
    fn apply(self, id: Entity, world: &mut World) {
        if !world.entities().contains(id) {
            #[cfg(feature="log")]
            bevy_log::warn!(entity=?id, "Tried to make entity an endpoint but it didn't exist");

            return; // Do nothing.
        }

        let runtime = world.resource::<crate::runtime::Runtime>();
        let comp = crate::endpoint::open(runtime.handle(), self.0);
        world.entity_mut(id).insert(comp);
    }
}

/// Opens a new connection with the target entity.
pub struct OpenConnection(OpenConnectionInner);

impl OpenConnection {
    pub fn advanced(
        endpoint: Entity,
        address: SocketAddr,
        config: quinn_proto::ClientConfig,
        hostname: impl Into<Arc<str>>,
    ) -> OpenConnection {
        let inner = OpenConnectionInner::Preconfigured {
            endpoint,
            address,
            config,
            hostname: hostname.into(),
        };

        return OpenConnection(inner);
    }
}

pub(crate) enum OpenConnectionInner {
    Preconfigured {
        endpoint: Entity,
        address: SocketAddr,
        config: quinn_proto::ClientConfig,
        hostname: Arc<str>,
    },
}

impl EntityCommand for OpenConnection {
    fn apply(self, id: Entity, world: &mut World) {
        todo!()
    }
}