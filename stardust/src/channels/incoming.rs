//! Incoming network message data attached to clients.

use std::collections::BTreeMap;
use bevy::prelude::*;
use crate::octets::payload::{Payloads, Payload};
use super::id::ChannelId;

/// Incoming messages from this remote peer.
/// 
/// Don't use this unless you are writing a transport layer.
/// Instead, use the `ChannelReader` systemparam.
#[derive(Component)]
pub struct IncomingNetworkMessages(BTreeMap<ChannelId, Payloads>);

impl IncomingNetworkMessages {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub(super) fn clear(&mut self) {
        self.0.clear()
    }

    pub fn append(&mut self, channel: ChannelId, payload: impl Into<Payload>) {
        let payloads = self.0.entry(channel).or_insert(Payloads::from(Vec::with_capacity(1)));
        payloads.0.push(payload.into());
    }

    pub fn read_all(&self) -> impl Iterator<Item = (&ChannelId, &Payloads)> {
        self.0.iter()
    }

    pub fn read_channel(&self, channel: ChannelId) -> Option<&Payloads> {
        self.0.get(&channel)
    }
}

impl std::fmt::Debug for IncomingNetworkMessages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IncomingNetworkMessages(field hidden)")
    }
}