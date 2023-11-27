//! Storage for newly received messages.

// Make the type system happy while simplifying the output of functions.
static EMPTY_OCTETSTRING_ARRAY: &[OctetString] = &[];

use std::collections::BTreeMap;
use bevy::prelude::*;
use crate::prelude::{ChannelId, OctetString};

/// Storage for network messages that have been received and directed to this peer.
#[derive(Component)]
pub struct IncomingMessageQueue(BTreeMap<ChannelId, Vec<OctetString>>);

impl IncomingMessageQueue {
    pub fn new() -> Self {
        Self(BTreeMap::default())
    }

    /// Returns a slice of all octet strings in the channel received by this peer.
    pub fn read_channel(&self, channel: ChannelId) -> &[OctetString] {
        match self.0.get(&channel) {
            Some(val) => &val,
            None => EMPTY_OCTETSTRING_ARRAY,
        }
    }

    /// Appends a message to the peer's message storage.
    pub fn append(&mut self, channel: ChannelId, octets: impl Into<OctetString>) {
        self.0
            .entry(channel)
            .or_insert(Vec::with_capacity(1))
            .push(octets.into());
    }
}