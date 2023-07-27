use std::{marker::PhantomData, collections::{BTreeMap, HashMap, hash_map::Iter}};
use bevy::{prelude::{Resource, Res, Entity, World}, ecs::system::SystemParam};
use crate::shared::{channel::{Channel, ChannelId}, protocol::Protocol};

/// Data from all channels.
#[derive(Resource)]
pub(super) struct AllChannelData(pub(crate) BTreeMap<ChannelId, ChannelData>);
impl AllChannelData {
    /// Gets all messages from all clients sent over the channel.
    fn get(&self, id: ChannelId) -> Option<&ChannelData> {
        self.0.get(&id)
    }
}

/// All messages from all clients.
pub struct ChannelData(pub(super) HashMap<Entity, Payloads>);
impl ChannelData {
    /// Returns all payloads from a single client.
    pub fn from_client(&self, client: Entity) -> Option<&Payloads> {
        self.0.get(&client)
    }

    /// Returns all [Payloads] from all clients, in arbitrary order. The [Payload]s will be ordered if the channel is ordered.
    pub fn all(&self) -> Iter<Entity, Payloads> {
        self.0.iter()
    }
}

/// All [Payload]s from a client. If the channel this originated from is ordered, the [Payload]s will be in order.
pub struct Payloads(pub(super) Box<[Payload]>);

/// A single network message sent over a channel, free of any additional transmission information when read.
pub struct Payload {
    pub(crate) ignore_head: usize,
    pub(crate) ignore_tail: usize,
    pub(crate) data: Box<[u8]>,
}

impl Payload {
    /// Gives access to the relevant octets of the message.
    pub fn read(&self) -> &[u8] {
        let len = self.data.len();
        &self.data[self.ignore_head-1..len - self.ignore_tail]
    }
}

/// Added to a Bevy system to read network messages over channel `T`.
#[derive(SystemParam)]
pub struct ChannelReader<'w, T: Channel> {
    store: Res<'w, AllChannelData>,
    protocol: Res<'w, Protocol>,
    phantom: PhantomData<T>,
}

impl<'w, T: Channel> ChannelReader<'w, T> {
    /// Accesses network messages. Will always return `None` if outside of `NetworkPreUpdate`.
    pub fn read(&self) -> Option<&ChannelData> {
        if self.store.0.is_empty() { return None; }
        let protocol = self.protocol.get_id::<T>()?;
        self.store.get(protocol)
    }
}

pub(super) fn clear_channel_data_system(
    world: &mut World,
) {
    world.resource_mut::<AllChannelData>().0.clear();
}