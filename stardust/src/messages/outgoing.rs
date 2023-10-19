//! Systemparams for transport layers to send messages from game systems.

use std::{sync::{RwLockReadGuard, Arc, RwLock}, marker::PhantomData};
use bevy::{prelude::*, ecs::system::SystemParam};
use smallvec::SmallVec;
use crate::prelude::{ChannelRegistry, ChannelId, OctetString, Channel};

#[derive(Debug, PartialEq, Eq)]
pub enum SendTarget {
    Single(Entity),
    Multiple(SmallVec<[Entity; 8]>),
    Broadcast,
}

#[derive(Resource)]
pub(crate) struct ChannelOctetStringCollectionArcHolder<T: Channel> {
    internal: Arc<RwLock<UntypedOctetStringCollection>>,
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

/// Allows transport layers to view all messages sent by game systems for transport.
#[derive(SystemParam)]
pub struct TransportOutgoingReader<'w> {
    registry: Res<'w, ChannelRegistry>,
}

impl<'w> TransportOutgoingReader<'w> {
    pub fn by_channel(&self, channel: ChannelId) /* -> impl Iterator<Item = ChannelIter<'w>> + 'w */ {
        todo!()
        // self.registry
        //     .get_outgoing_arc_map()
        //     .iter()
        //     .map(|(k,v)| {
        //         ChannelIter {
        //             channel: *k,
        //             guard: v.read().unwrap()
        //         }
        //     })
    }
}

pub struct ChannelIter<'a> {
    channel: ChannelId,
    guard: RwLockReadGuard<'a, UntypedOctetStringCollection>
}

pub(crate) struct UntypedOctetStringCollection(Vec<(SendTarget, OctetString)>);

impl UntypedOctetStringCollection {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}