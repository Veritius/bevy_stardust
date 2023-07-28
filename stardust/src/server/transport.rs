//! Types for writing custom transport layers.

use bevy::prelude::*;
use crate::shared::serialisation::Octet;

/// All packets waiting to be sent to clients.
#[derive(Resource)]
pub struct WaitingPackets {
    targets: Box<[(Box<[Entity]>, usize)]>,
    payloads: Box<[Box<[Octet]>]>,
}

impl WaitingPackets {
    /// Returns an iterator that gives all targets and octets for transport as the Item.
    pub fn all(&self) -> WaitingPacketReader {
        WaitingPacketReader {
            pkts: &self,
            targets_idx: 0,
        }
    }
}

/// An iterator that gives all targets for a single set of octets as the `Item` value.
/// The way to use this is for each `Item`, you send all the octets to the 
pub struct WaitingPacketReader<'a> {
    pkts: &'a WaitingPackets,
    targets_idx: usize,
}

impl<'a> Iterator for WaitingPacketReader<'a> {
    type Item = (&'a [Entity], &'a [Octet]);

    fn next(&mut self) -> Option<Self::Item> {
        let (targets, payload_idx) = self.pkts.targets.get(self.targets_idx)?;
        self.targets_idx += 1;
        Some((&targets, &self.pkts.payloads[*payload_idx]))
    }
}