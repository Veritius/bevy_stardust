use bevy_ecs::{prelude::*, system::EntityCommands};

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
    commands: EntityCommands<'a>,
}