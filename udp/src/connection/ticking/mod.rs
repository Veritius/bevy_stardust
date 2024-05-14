mod recv_stardust;

use bevy_stardust::prelude::*;
use ticking::handshake::HandshakeOutcome;
use crate::plugin::PluginConfiguration;
use self::packets::{frames::FrameType, reader::PacketReaderContext};
use super::*;

impl ConnectionInner {
    /// Ticks the connection, parsing incoming data.
    pub(super) fn tick_preupdate(&mut self, mut context: PreUpdateTickData) {
        if let Some(handshake) = &mut self.handshake {
            while let Some(packet) = self.recv_queue.pop_front() {
                let outcome = handshake.recv(
                    packet,
                    context.config,
                    &mut self.reliability,
                    &mut self.unacked_pkts
                );

                match outcome {
                    None => {},
                    Some(HandshakeOutcome::FinishedHandshake) => todo!(),
                    Some(HandshakeOutcome::FailedHandshake) => todo!(),
                }
            }
        }

        // Iterator to parse all incoming packets
        let mut frames = self.frame_parser.iter(PacketReaderContext {
            queue: &mut self.recv_queue,
            config: context.config,
            reliability: &mut self.reliability,
            rel_packets: &mut self.unacked_pkts
        });

        // Read frames from the iterator until we run out
        while let Some(frame) = frames.next() {
            match frame {
                Ok(frame) => {
                    match frame.ftype {
                        FrameType::Control => todo!(),
                        FrameType::Stardust => todo!(),
                    }
                },
                Err(error) => {
                    todo!()
                },
            }
        }
    }

    /// Ticks the connection, queuing outgoing data.
    pub(super) fn tick_postupdate(&mut self, context: PostUpdateTickData) {

    }
}

/// Data used by [`tick_preupdate`](ConnectionInner::tick_preupdate)
pub(super) struct PreUpdateTickData<'a> {
    pub config: &'a PluginConfiguration,
    pub registry: &'a ChannelRegistryInner,
    pub messages: Option<Mut<'a, NetworkMessages<Incoming>>>,
}

/// Data used by [`tick_postupdate`](ConnectionInner::tick_postupdate)
pub(super) struct PostUpdateTickData<'a> {
    pub config: &'a PluginConfiguration,
    pub registry: &'a ChannelRegistryInner,
    pub messages: Option<Ref<'a, NetworkMessages<Outgoing>>>,
}