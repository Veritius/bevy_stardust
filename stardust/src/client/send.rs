//! Sending systemparams for the client.

use bevy::{prelude::*, ecs::system::SystemParam};
use crate::shared::{channels::{id::Channel, outgoing::{OutgoingOctetStrings, SendTarget}}, octetstring::OctetString};
use super::peers::Server;

#[derive(SystemParam)]
pub struct ChannelWriter<'w, 's, T: Channel> {
    server: Query<'w, 's, Entity, With<Server>>,
    writer: ResMut<'w, OutgoingOctetStrings<T>>,
}

impl<'w, 's, T: Channel> ChannelWriter<'w, 's, T> {
    pub fn send(&mut self, octets: impl Into<OctetString>) -> Result<(), ChannelWritingError> {
        if self.server.is_empty() { return Err(ChannelWritingError::NotConnected) };
        let server = self.server.single();
        self.writer.send(SendTarget::Single(server), octets);
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChannelWritingError {
    NotConnected,
}