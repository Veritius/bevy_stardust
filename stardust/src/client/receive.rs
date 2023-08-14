use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::shared::{channels::{registry::ChannelRegistry, id::Channel, incoming::IncomingNetworkMessages}, payload::Payloads};
use super::peers::Server;

/// Added to a Bevy system to read network messages over channel `T`.
#[derive(SystemParam)]
pub struct ChannelReader<'w, 's, T: Channel> {
    server: Query<'w, 's, &'static IncomingNetworkMessages, With<Server>>,
    channel_registry: Res<'w, ChannelRegistry>,
    phantom: PhantomData<T>,
}

impl<'w, 's, T: Channel> ChannelReader<'w, 's, T> {
    pub fn read_from_server(&self) -> Result<Option<&Payloads>, ChannelReadingError> {
        if self.server.is_empty() { return Err(ChannelReadingError::NotConnected); }
        let val = &self.channel_registry.get_from_type::<T>()
            .expect("Tried to access the data of channel T but it was not registered!");
        Ok(self.server
            .single()
            .0.get(&val.0))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChannelReadingError {
    NotConnected,
    NoMessages,
}