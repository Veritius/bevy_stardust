//! Storage for newly received messages.

// Make the type system happy while simplifying the output of functions.
static EMPTY_OCTETSTRING_ARRAY: &[OctetString] = &[];

use std::collections::BTreeMap;
use bevy::prelude::*;
use crate::{prelude::{ChannelId, OctetString}, scheduling::NetworkScheduleData};

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
    /// Panics if this is done outside of the `TransportReadPackets` schedule.
    pub fn append(&mut self, schedule: &NetworkScheduleData, channel: ChannelId, octets: impl Into<OctetString>) {
        if !schedule.message_storage_mutation_allowed() {
            panic!("Tried to append a message outside of the TransportReadPackets schedule");
        }

        self.0
            .entry(channel)
            .or_insert(Vec::with_capacity(1))
            .push(octets.into());
    }
}