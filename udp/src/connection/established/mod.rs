mod closing;
mod control;
mod frames;
mod polling;
mod writer;

use closing::Closing;
use control::ControlFrame;
use smallvec::SmallVec;

pub(super) use polling::established_reading_system;
pub(super) use writer::{
    established_resend_system,
    established_writing_system,
};
pub(super) use control::established_control_system;
pub(super) use closing::{
    DisconnectEstablishedPeerEvent,
    established_close_events_system,
    established_closing_write_system,
    established_close_despawn_system,
};

use bevy::prelude::*;
use super::{ordering::OrderingManager, reliability::{ReliabilityState, ReliablePackets}};
use frames::{builder::PacketBuilder, reader::PacketParser};

#[derive(Component)]
pub(crate) struct Established {
    reliability: ReliablePackets,
    orderings: OrderingManager,

    reader: PacketParser,
    builder: PacketBuilder,
    control: SmallVec<[ControlFrame; 2]>,

    closing: Option<Closing>,
    ice_thickness: u16,
}

impl Established {
    pub(in super::super) fn new(
        reliability: ReliabilityState,
    ) -> Self {
        Self {
            reliability: ReliablePackets::new(reliability.clone()),
            orderings: OrderingManager::new(),

            reader: PacketParser::default(),
            builder: PacketBuilder::default(),
            control: SmallVec::new(),

            closing: None,
            ice_thickness: u16::MAX,
        }
    }
}