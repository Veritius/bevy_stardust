mod control;
mod ordering;
mod packets;
mod postupdate;
mod preupdate;
mod stardust;

use std::collections::BTreeMap;
use crate::sequences::SequenceId;
use self::{control::ConnectionController, ordering::OrderingManager, packets::{builder::PacketBuilder, reader::PacketReader}};
use super::{handshake::HandshakeStateMachine, machine::{PostUpdateTickData, PreUpdateTickData}, reliability::UnackedPacket, shared::ConnectionShared};

/// State machine for established connections.
pub(super) struct EstablishedStateMachine {
    controller: ConnectionController,
    orderings: OrderingManager,
    unacked_pkts: BTreeMap<SequenceId, UnackedPacket>,

    frame_builder: PacketBuilder,
    frame_parser: PacketReader,
}

impl EstablishedStateMachine {
    pub fn new(_handshake: HandshakeStateMachine) -> Self {
        Self {
            controller: ConnectionController::default(),
            orderings: OrderingManager::new(),
            unacked_pkts: BTreeMap::new(),

            frame_builder: PacketBuilder::default(),
            frame_parser: PacketReader::default(),
        }
    }
}