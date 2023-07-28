use std::{marker::PhantomData, collections::{BTreeMap, HashMap, hash_map::Iter}};
use bevy::{prelude::{Resource, Res, Entity, World}, ecs::system::SystemParam};
use crate::shared::{receive::Payloads, channels::{id::{ChannelId, Channel}, registry::ChannelRegistry}};

/// Data from all channels.
#[derive(Resource, Default)]
pub(super) struct AllChannelData(pub(crate) BTreeMap<ChannelId, AllClientMessages>);
impl AllChannelData {
    /// Gets all messages from all clients sent over the channel.
    fn get(&self, id: ChannelId) -> Option<&AllClientMessages> {
        self.0.get(&id)
    }
}

/// All messages from all clients from a specific channel.
pub struct AllClientMessages(pub(super) HashMap<Entity, Payloads>);
impl AllClientMessages {
    /// Returns all payloads from a single client.
    pub fn from_client(&self, client: Entity) -> Option<&Payloads> {
        self.0.get(&client)
    }

    /// Returns all [Payloads] from all clients, in arbitrary order. The [Payload]s will be ordered if the channel is ordered.
    pub fn all(&self) -> Iter<Entity, Payloads> {
        self.0.iter()
    }
}

/// Added to a Bevy system to read network messages over channel `T`.
#[derive(SystemParam)]
pub struct ChannelReader<'w, T: Channel> {
    store: Res<'w, AllChannelData>,
    channel_registry: Res<'w, ChannelRegistry>,
    phantom: PhantomData<T>,
}

impl<'w, T: Channel> ChannelReader<'w, T> {
    /// Accesses network messages. Will always return `None` if outside of `NetworkPreUpdate`.
    pub fn read(&self) -> Option<&AllClientMessages> {
        todo!()
    }
}

pub(super) fn clear_channel_data_system(
    world: &mut World,
) {
    world.resource_mut::<AllChannelData>().0.clear();
}