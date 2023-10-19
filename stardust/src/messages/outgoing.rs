//! Systemparams for transport layers to send messages from game systems.

use std::{sync::{RwLockReadGuard, Arc, RwLock}, marker::PhantomData};
use bevy::{prelude::*, ecs::system::SystemParam};
use smallvec::SmallVec;
use crate::prelude::{ChannelRegistry, ChannelId, OctetString, Channel};

/// Value used to specify which peers should receive a network message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendTarget {
    /// Sends to one peer.
    Single(Entity),
    /// Sends to multiple peers.
    // TODO: Remove in favor of network groups.
    Multiple(SmallVec<[Entity; 8]>),
    /// Sends to all peers.
    Broadcast,
}

#[derive(Resource)]
pub(crate) struct ChannelOctetStringCollectionArcHolder<T: Channel> {
    pub internal: Arc<RwLock<UntypedOctetStringCollection>>,
    phantom: PhantomData<T>,
}

impl<T: Channel> ChannelOctetStringCollectionArcHolder<T> {
    pub fn new(store: UntypedOctetStringCollection) -> Self {
        Self {
            internal: Arc::new(RwLock::new(store)),
            phantom: PhantomData,
        }
    }
}

/// Allows transport layers to view all messages queued for transport by game systems.
#[derive(SystemParam)]
pub struct TransportOutgoingReader<'w> {
    registry: Res<'w, ChannelRegistry>,
}

impl<'w> TransportOutgoingReader<'w> {
    /// Returns an iterator over all channels, with the collection of messages.
    pub fn by_channel(&'w self, channel: ChannelId) -> impl Iterator<Item = ChannelReader<'w>> + 'w {
        self.registry
            .get_outgoing_arc_map()
            .iter()
            .filter(move |(cid, _)| **cid == channel)
            .map(|(k,v)| {
                ChannelReader {
                    channel: *k,
                    guard: v.read().unwrap()
                }
            })
    }
}

/// Read lock on a channel's collection of octet strings.
pub struct ChannelReader<'a> {
    channel: ChannelId,
    guard: RwLockReadGuard<'a, UntypedOctetStringCollection>
}

impl<'a> ChannelReader<'a> {
    /// Returns the channel ID corresponding to this ChannelReader.
    pub fn channel(&self) -> ChannelId {
        self.channel
    }

    /// Returns a slice of all messages.
    pub fn read(&self) -> &[(SendTarget, OctetString)] {
        &self.guard.0
    }
}

#[derive(Clone)]
pub(crate) struct UntypedOctetStringCollection(pub(super) Vec<(SendTarget, OctetString)>);

impl UntypedOctetStringCollection {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}