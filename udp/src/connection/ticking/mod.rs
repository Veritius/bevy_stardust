mod recv_stardust;

use bevy_stardust::prelude::*;
use crate::plugin::PluginConfiguration;
use self::packets::{frames::FrameType, reader::PacketReaderContext};
use super::*;

impl ConnectionInner {
    /// Ticks the connection, parsing incoming data.
    pub(super) fn tick_preupdate(&mut self, mut context: PreUpdateTickData) {
        match &mut self.state {
            ConnectionStateInner::Handshaking { machine } => {
                todo!()
            },

            ConnectionStateInner::Established => {
                // Iterator to parse all incoming packets
                let mut frames = self.frame_parser.iter(PacketReaderContext {
                    queue: &mut self.recv_queue,
                    config: context.config,
                    reliability: &mut self.reliability,
                    rel_packets: &mut self.unacked_pkts,
                });

                // Loop over all frames
                while let Some(frame) = frames.next() {
                    match frame {
                        // Case 1: Frame read
                        Ok(frame) => {
                            match frame.ftype {
                                // Case 1.1: Connection control frame
                                FrameType::Control => todo!(),
                                // Case 1.2: Stardust message frame
                                FrameType::Stardust => {
                                    // TODO: Don't panic here, handle it somehow.
                                    // TODO: This goes through two separate pointers to access the actual data, fix that.
                                    let messages = context.messages.as_mut().unwrap();

                                    // Pass the frame to a distinct function to not clutter this one
                                    if let Err(error) = recv_stardust::recv_stardust_frame(
                                        context.registry,
                                        &mut self.orderings,
                                        messages,
                                        frame,
                                    ) {
                                        todo!()
                                    }
                                },
                            }
                        },

                        // Case 2: An error occurred
                        Err(error) => {
                            todo!()
                        },
                    }
                }
            },

            ConnectionStateInner::Closing => {
                todo!()
            },

            ConnectionStateInner::Closed => {
                /*
                    The connection is closed, do nothing.
                    What were you expecting?
                */
            },
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