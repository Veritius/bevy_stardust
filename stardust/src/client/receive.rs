use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::shared::{channels::{registry::ChannelRegistry, id::Channel}, receive::Payloads, messages::receive::IncomingNetworkMessages};
use super::peers::Server;

/// Added to a Bevy system to read network messages over channel `T`.
#[derive(SystemParam)]
pub struct ChannelReader<'w, 's, T: Channel> {
    server: Query<'w, 's, &'static IncomingNetworkMessages, With<Server>>,
    channel_registry: Res<'w, ChannelRegistry>,
    phantom: PhantomData<T>,
}

impl<'w, 's, T: Channel> ChannelReader<'w, 's, T> {
    pub fn read_from_server(&self) -> Option<&Payloads> {
        let val = &self.channel_registry.get_from_type::<T>();
        if val.is_none() { return None; }
        self.server
            .single()
            .0.get(&val.unwrap().0)
    }
}