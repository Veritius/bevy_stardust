use std::{marker::PhantomData, sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard}};
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::shared::{octetstring::OctetString, channels::id::Channel};
use super::{registry::ChannelRegistry, id::ChannelId};

// TODO: This is pretty janky, clean it up a bit.

/// SystemParam that allows reading `OutgoingOctetStringsUntyped`. Accesses are read-only and can be used in parallel.
#[derive(SystemParam)]
pub struct OutgoingOctetStringsAccessor<'w> {
    registry: Res<'w, ChannelRegistry>,
}

impl OutgoingOctetStringsAccessor<'_> {
    /// Returns an iterator that only returns octet strings that should be sent to a specific client.
    pub fn by_client(&self, client: Entity) -> impl Iterator<Item = &OctetString> {
        make_by_client_iter(self, client)
    }

    /// Returns an iterator that returns send targets and octet strings by channel.
    pub fn by_channel(&self) -> impl Iterator<Item = OutgoingOctetStringAccessorItem> + '_ {
        struct OutgoingOctetStringsAccessorChannelIterator<'a> {
            registry: &'a ChannelRegistry,
            index: u32,
        }

        impl<'a> Iterator for OutgoingOctetStringsAccessorChannelIterator<'a> {
            type Item = OutgoingOctetStringAccessorItem;

            fn next(&mut self) -> Option<Self::Item> {
                if self.index >= self.registry.channel_count() { return None; }
                let id = ChannelId::try_from(self.index);
                self.index += 1;

                if id.is_err() { return None; }
                let id = id.unwrap();
                let arc = self.registry.get_outgoing_arc(id)?.clone();

                Some(OutgoingOctetStringAccessorItem { id, arc })
            }
        }

        OutgoingOctetStringsAccessorChannelIterator {
            registry: &self.registry,
            index: 0,
        }
    }
}

pub struct OutgoingOctetStringAccessorItem {
    id: ChannelId,
    arc: Arc<RwLock<OutgoingOctetStringsUntyped>>,
}

impl OutgoingOctetStringAccessorItem {
    pub fn id(&self) -> ChannelId {
        self.id
    }

    pub fn read<'a>(&'a self) -> RwLockReadGuard<'a, OutgoingOctetStringsUntyped> {
        self.arc.read().unwrap()
    }

    pub fn write<'a>(&'a mut self) -> RwLockWriteGuard<'a, OutgoingOctetStringsUntyped> {
        self.arc.write().unwrap()
    }
}

/// Used to write octet strings to remote peers. No associated type or channel information, and only accessible in `TransportSendPackets` for use by transport layers.
/// 
/// This is only returned for use in transport layers. Use the `ChannelWriter<T>` systemparam.
pub struct OutgoingOctetStringsUntyped(Vec<(SendTarget, OctetString)>);

impl OutgoingOctetStringsUntyped {
    pub(in crate::shared) fn new() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self(vec![])))
    }

    pub(in crate) fn send(&mut self, target: SendTarget, octets: impl Into<OctetString>) {
        self.0.push((target, octets.into()))
    }

    pub(in crate::shared) fn clear(&mut self) {
        self.0.clear()
    }

    /// Counts how many messages need sending
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// Creates an iterator over octets and targets to send them to.
    pub fn read(&self) -> impl Iterator<Item = &(SendTarget, OctetString)> {
        self.0.iter()
    }
}

/// Used to write octet strings to remote peers.
/// You should use the client/server variations of `ChannelReader` and `ChannelWriter`
#[derive(Resource)]
pub struct OutgoingOctetStrings<T: Channel> {
    internal: Arc<RwLock<OutgoingOctetStringsUntyped>>,
    phantom: PhantomData<T>,
}

impl<T: Channel> OutgoingOctetStrings<T> {
    pub(in crate::shared) fn new(internal: Arc<RwLock<OutgoingOctetStringsUntyped>>) -> Self {
        Self {
            internal,
            phantom: PhantomData,
        }
    }

    /// Returns the internal RwLock, for uses in concurrency and multi-threading.
    pub fn get_lock(&mut self) -> &RwLock<OutgoingOctetStringsUntyped> {
        &self.internal
    }

    /// Sends `octets to `target` over channel `T`
    /// 
    /// This function intentionally requires `&mut self`, even when it isn't technically necessary, to prevent blocking.
    /// If you want to use this in a multi-threaded context, use `get_lock`.
    pub fn send(&mut self, target: SendTarget, octets: impl Into<OctetString>) {
        self.internal.write().unwrap().send(target, octets);
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SendTarget {
    /// Sends the message to a single peer.
    Single(Entity),
    /// Sends the message to multiple peers.
    Multiple(Box<[Entity]>),
    /// Sends the message to all peers.
    Broadcast,
}

impl SendTarget {
    /// Returns `true` if `client` is included in `self`.
    pub fn includes(&self, client: Entity) -> bool {
        match self {
            SendTarget::Single(val) => { *val == client },
            SendTarget::Multiple(vals) => { vals.contains(&client) },
            SendTarget::Broadcast => { true },
        }
    }

    /// Returns `true` if `client` is not included in `self`.
    pub fn excludes(&self, client: Entity) -> bool {
        !self.includes(client)
    }
}

impl From<Entity> for SendTarget {
    fn from(value: Entity) -> Self {
        Self::Single(value)
    }
}

impl TryFrom<&[Entity]> for SendTarget {
    type Error = ();
    fn try_from(value: &[Entity]) -> Result<Self, Self::Error> {
        match value.len() {
            0 => Err(()),
            1 => Ok(Self::Single(value[0].clone())),
            _ => Ok(Self::Multiple(value.iter().cloned().collect::<Box<[Entity]>>()))
        }
    }
}

/// Makes an iterator that accesses all octet strings, filtered by client.
fn make_by_client_iter<'a>(accessor: &'a OutgoingOctetStringsAccessor, client: Entity) -> impl Iterator<Item = &'a OctetString> + 'a {
    struct OutgoingOctetStringClientIterator<'a> {
        target: Entity,
        registry: &'a ChannelRegistry,
        channel_idx: u32,
    }

    impl<'a> Iterator for OutgoingOctetStringClientIterator<'a> {
        type Item = &'a OctetString;

        fn next(&mut self) -> Option<Self::Item> {
            let channel_id = TryInto::<ChannelId>::try_into(self.channel_idx).unwrap();

            todo!()
        }
    }

    OutgoingOctetStringClientIterator {
        target: client,
        registry: &accessor.registry,
        channel_idx: 0,
    }
}