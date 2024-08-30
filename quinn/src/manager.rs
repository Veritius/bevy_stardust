use bevy_ecs::{prelude::*, system::EntityCommands};
use quinn_proto::ClientConfig;

pub trait EndpointCommands {
    fn make_endpoint(
        &mut self,
        build: impl FnOnce(EndpointBuilder),
    ) -> &mut Self;

    fn close_endpoint(
        &mut self,
    ) -> &mut Self;
}

impl<'w> EndpointCommands for EntityWorldMut<'w> {
    fn make_endpoint(
        &mut self,
        build: impl FnOnce(EndpointBuilder),
    ) -> &mut Self {
        todo!()
    }

    fn close_endpoint(
        &mut self,
    ) -> &mut Self {
        todo!()
    }
}

impl<'w> EndpointCommands for EntityCommands<'w> {
    fn make_endpoint(
        &mut self,
        build: impl FnOnce(EndpointBuilder),
    ) -> &mut Self {
        todo!()
    }

    fn close_endpoint(
        &mut self,
    ) -> &mut Self {
        todo!()
    }
}

pub struct EndpointBuilder<'a> {
    commands: &'a mut quinn_proto::Endpoint,
}

impl<'a> EndpointBuilder<'a> {
    pub fn connect(
        &mut self,
        config: ClientConfig,
        build: impl FnOnce(EndpointBuilder),
    ) {
        todo!()
    }
}

pub struct ConnectionBuilder<'a> {
    commands: EntityCommands<'a>,
}