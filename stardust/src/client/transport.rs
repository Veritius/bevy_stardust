pub mod udp;

use bevy::prelude::*;
use crate::shared::serialisation::Octet;

/// All packets waiting to be sent to the server.
#[derive(Resource)]
pub struct WaitingPackets {
    payloads: Box<[Box<[Octet]>]>,
}

impl WaitingPackets {
    /// Returns all payloads for sending to the server.
    pub fn all(&self) -> &[Box<[Octet]>] {
        &self.payloads
    }
}