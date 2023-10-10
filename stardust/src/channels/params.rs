//! Channel-related parameters.

use std::marker::PhantomData;
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::prelude::{Channel, ChannelRegistry, OctetString};
use super::incoming::IncomingNetworkMessages;
use super::outgoing::{OutgoingNetworkMessages, SendTarget};

/// Allows writing octets to channel `T`.
#[derive(SystemParam)]
pub struct NetworkWriter<'w, 's, T: Channel> {
    outgoing: ResMut<'w, OutgoingNetworkMessages<T>>,
    phantom: PhantomData<&'s ()>,
}

impl<'w, 's, T: Channel> NetworkWriter<'w, 's, T> {
    /// Send one octet string.
    pub fn send(&mut self, target: SendTarget, octets: impl Into<OctetString>) {
        self.outgoing.send(target, octets);
    }

    /// Send an iterator of octet strings.
    /// For large amounts of octet strings, this is more performant.
    pub fn send_batch(&mut self, strings: impl Iterator<Item = (SendTarget, impl Into<OctetString>)>) {
        self.outgoing.send_batch(strings);
    }
}

/// Allows reading octets from channel `T`.
#[derive(SystemParam)]
pub struct NetworkReader<'w, 's, T: Channel> {
    incoming: Query<'w, 's, &'static IncomingNetworkMessages>,
    registry: Res<'w, ChannelRegistry>,
    phantom: PhantomData<T>,
}

impl<'w, 's, T: Channel> NetworkReader<'w, 's, T> {
    
}