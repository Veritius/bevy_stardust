use bevy::prelude::*;
use crate::shared::octetstring::OctetString;

/// All octet strings waiting to be sent to the server.
#[derive(Resource)]
pub struct WaitingPackets {
    payloads: Box<[OctetString]>,
}

impl WaitingPackets {
    /// Returns all payloads for sending to the server.
    pub fn all(&self) -> &[OctetString] {
        &self.payloads
    }
}