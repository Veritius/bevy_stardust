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

/// Creates a new QUIC connection based on an endpoint.
pub struct OpenConnection {

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