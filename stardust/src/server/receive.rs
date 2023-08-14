use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::shared::{channels::{id::Channel, registry::ChannelRegistry, incoming::IncomingNetworkMessages}, payload::{Payloads, Payload}};
use super::clients::Client;

/// Added to a Bevy system to read network messages over channel `T`.
#[derive(SystemParam)]
pub struct ChannelReader<'w, 's, T: Channel> {
    channel_registry: Res<'w, ChannelRegistry>,
    clients: Query<'w, 's, (Entity, &'static IncomingNetworkMessages), With<Client>>,
    phantom: PhantomData<T>,
}

impl<'w, 's, T: Channel> ChannelReader<'w, 's, T> {
    /// Reads messages on this channel from a specific client.
    /// 
    /// Panics if `T` doesn't exist.
    pub fn read_client(&self, client: Entity) -> Result<Option<&Payloads>, ChannelReadingError> {
        // Get channel data
        let (channel_id, _) = self.channel_registry.get_from_type::<T>()
            .expect("Tried to access the data of channel T but it was not registered!");

        // Get client data
        let qres = self.clients.get(client);
        let Ok((_, data)) = qres else { return Err(ChannelReadingError::NonexistentClient) };

        // Return data
        return Ok(data.0.get(&channel_id))
    }

    /// Reads messages on this channel from all clients, including their entity ID in the iterator.
    pub fn read_all(&self) -> impl Iterator<Item = (Entity, &Payloads)> {
        let (channel_id, _) = self.channel_registry.get_from_type::<T>()
            .expect("Tried to access the data of channel T but it was not registered!");
        let client_iter = self.clients.iter();
        
        client_iter
            .map(move |(e, v)| (e, v.0.get(&channel_id)))
            .filter(|(_, v)| v.is_some())
            .map(|(e, v)| (e, v.unwrap()))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChannelReadingError {
    NonexistentClient,
}