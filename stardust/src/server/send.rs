//! Sending systemparams for the server.

use std::marker::PhantomData;
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::shared::channels::id::Channel;
use crate::shared::channels::outgoing::{OutgoingOctetStrings, SendTarget};
use crate::shared::octetstring::OctetString;

/// Sends messages over a network channel to clients.
#[derive(SystemParam)]
pub struct ChannelWriter<'w, T: Channel> {
    outgoing: ResMut<'w, OutgoingOctetStrings<T>>,
    phantom: PhantomData<T>,
}

impl<'w, T: Channel> ChannelWriter<'w, T> {
    pub fn send(&mut self, target: SendTarget, octets: impl Into<OctetString>) -> Result<(), ChannelWritingError> {
        self.outgoing.send(target, octets);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChannelWritingError {
    NonexistentClient,
}