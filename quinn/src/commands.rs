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

    /// Makes the target entity a connection, sugaring [`OpenConnection`].
    /// 
    /// Fails if the entity does not exist, or is already a connection.
    fn open_connection(
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
    fn open_connection(
        &mut self,
        config: OpenConnection,
    ) -> &mut Self {
        self.add(config);
        return self;
    }
}

/// Opens a new endpoint with the target entity.
pub struct MakeEndpoint {

}

impl EntityCommand for MakeEndpoint {
    fn apply(
        self,
        id: Entity,
        world: &mut World,
    ) {
        todo!()
    }
}

/// Opens a new connection with the target entity.
pub struct OpenConnection {
    /// The entity ID of the endpoint to use for I/O.
    pub endpoint: Entity,
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