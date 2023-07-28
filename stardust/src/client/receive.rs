use std::{marker::PhantomData, collections::BTreeMap};
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::shared::{channel::{Channel, ChannelId}, protocol::Protocol, receive::Payloads};

#[derive(Resource, Default)]
pub(super) struct AllChannelData(pub(crate) BTreeMap<ChannelId, Payloads>);

impl AllChannelData {
    pub fn get(&self, channel: ChannelId) -> Option<&Payloads> {
        self.0.get(&channel)
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
    pub fn read(&self) -> Option<&Payloads> {
        if self.store.0.is_empty() { return None; }
        let id = self.protocol.get_id::<T>()?;
        self.store.get(id)
    }
}