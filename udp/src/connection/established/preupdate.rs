use self::packets::{frames::FrameType, reader::PacketReaderContext};
use super::*;

impl EstablishedStateMachine {
    pub fn tick_preupdate(
        &mut self,
        shared: &mut ConnectionShared,
        mut context: PreUpdateTickData,
    ) {
        // Message storage thing. It should exist by this point, so we can unwrap.
        // Note: further derefs trigger change detection, which is why the type signature
        // of this variable is the rather odd &mut Mut<NetworkMessages<Incoming>>
        let messages = context.messages.as_mut().unwrap();

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
                        FrameType::Stardust => {
                            match stardust::read_stardust_frame(
                                frame,
                                context.registry,
                                &mut self.orderings,
                            ) {
                                Ok((channel, payload)) => {
                                    // Add to the message to the queue
                                    messages.push(channel, payload);
                                },
                                Err(_) => {
                                    todo!()
                                },
                            }
                        },
                    }
                },
                Err(err) => {
                    todo!()
                },
            }
        }
    }
}