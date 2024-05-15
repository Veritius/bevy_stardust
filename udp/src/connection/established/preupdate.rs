use bevy_stardust::channels::ChannelId;

use self::packets::{frames::FrameType, reader::PacketReaderContext};
use super::*;

impl EstablishedStateMachine {
    pub fn tick_preupdate(
        &mut self,
        shared: &mut ConnectionShared,
        context: PreUpdateTickData,
    ) {
        // Iterator that reads frames inside packets
        let mut parser = self.frame_parser.iter(PacketReaderContext {
            queue: &mut shared.recv_queue,
            config: context.config,
            reliability: &mut shared.reliability,
            rel_packets: &mut self.unacked_pkts,
        });

        // Iterate over all frames
        while let Some(frame) = parser.next() {
            match frame {
                Ok(frame) => {
                    match frame.ftype {
                        FrameType::Control => todo!(),
                        FrameType::Stardust => todo!(),
                    }
                },
                Err(err) => {
                    todo!()
                },
            }
        }
    }
}