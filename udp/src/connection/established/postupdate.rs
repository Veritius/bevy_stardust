use std::time::Instant;

use bevy_stardust::prelude::*;
use crate::varint::VarInt;

use self::packets::{builder::PacketBuilderContext, frames::{FrameFlags, FrameType, SendFrame}};
use super::*;

impl EstablishedStateMachine {
    pub fn tick_postupdate(
        &mut self,
        shared: &mut ConnectionShared,
        context: PostUpdateTickData,
    ) {
        // Message storage thing. It should exist by this point, so we can unwrap.
        // Note: further derefs trigger change detection, which is why the type signature
        // of this variable is the rather odd &mut Mut<NetworkMessages<Outgoing>>
        let messages = context.messages.as_ref().unwrap();

        // Add all messages to the builder
        // This won't repeat anything since the messages store is cleared each tick
        for (channel, messages) in messages.iter() {
            let channel_data = context.registry.channel_config(channel).unwrap();

            let is_ordered = channel_data.ordered != OrderingGuarantee::Unordered;
            let is_reliable = channel_data.reliable == ReliabilityGuarantee::Reliable;
            
            let mut flags = FrameFlags::IDENTIFIED;
            if is_ordered { flags |= FrameFlags::ORDERED }

            match is_ordered {
                true => {
                    let orderings = self.orderings.get(channel_data);

                    for payload in messages.iter().cloned() {
                        self.frame_builder.put(SendFrame {
                            priority: channel_data.priority,
                            time: Instant::now(),
                            flags,
                            ftype: FrameType::Stardust,
                            reliable: is_reliable,
                            order: Some(orderings.advance()),
                            ident: Some(VarInt::from_u32(channel_data.channel_id.into())),
                            payload,
                        });
                    }
                },

                false => {
                    for payload in messages.iter().cloned() {
                        self.frame_builder.put(SendFrame {
                            priority: channel_data.priority,
                            time: Instant::now(),
                            flags,
                            ftype: FrameType::Stardust,
                            reliable: is_reliable,
                            order: None,
                            ident: Some(VarInt::from_u32(channel_data.channel_id.into())),
                            payload,
                        });
                    }
                },
            }
        }

        let mut scratch = Vec::with_capacity(shared.mtu_limit);
        let mut frames = self.frame_builder.run(
            shared.budget_limit,
            shared.mtu_limit,
            PacketBuilderContext {
                config: context.config,
                rel_state: &mut shared.reliability,
                rel_packets: &mut self.unacked_pkts,
                scratch: &mut scratch,
            }
        );
        drop(scratch);

        for frame in frames.drain(..) {
            shared.send_queue.push_back(frame);
        }
    }
}