use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{connection::{ordering::OrderedMessage, packets::{frames::FrameType, reader::PacketReaderContext}}, plugin::PluginConfiguration, prelude::*};
use super::{control::*, Established};

impl Established {
    pub(super) fn poll(
        &mut self,
        connection: &mut Connection,
        mut messages: Mut<NetworkMessages<Incoming>>, // because change detection
        registry: &ChannelRegistryInner,
        config: &PluginConfiguration,
    ) {
        // Context object for the packet reader
        let context = PacketReaderContext {
            queue: &mut connection.recv_queue,
            config: &config,
            reliability: &mut self.reliability,
        };

        // Iterate over all frames
        // This runs until there is no more data to parse
        let mut iter = self.reader.iter(context);
        'frames: loop {
            match iter.next() {
                // Case 1: Another frame was read
                Some(Ok(frame)) => {
                    match frame.ftype {
                        // Case 1.1: Connection control frame
                        FrameType::Control => {
                            // Unwrapping is fine since the parser checks it
                            let ident = frame.ident.unwrap();

                            // TODO: Figure out a better solution than this
                            match ident {
                                CTRL_ID_CLOSE => { connection.state = ConnectionState::Closed; },

                                // Unrecognised identifier
                                _ => { self.ice_thickness = self.ice_thickness.saturating_sub(400); }
                            }
                        },

                        // Case 1.2: Stardust message frame
                        FrameType::Stardust => {
                            // Unwrapping is ok since the presence of ident is checked in the parser
                            // It's also checked to be within 2^32 (max channel ids) so we can unwrap there too
                            let channel: ChannelId = frame.ident.unwrap().try_into().unwrap();

                            // Quickly check that the channel exists
                            let channel_data = match registry.channel_config(channel) {
                                Some(v) => v,
                                None => {
                                    // Channel doesn't exist
                                    todo!();
                                },
                            };

                            match channel_data.ordered != OrderingGuarantee::Unordered {
                                // Case 1.2.1: Ordered message
                                true => {
                                    // Ensure that we have an ordering id to use
                                    if frame.order.is_none() { todo!() }
                                    let sequence = frame.order.unwrap();

                                    // Ordering state for this channel
                                    let ordering = self.orderings.get(channel_data);

                                    // Receive the ordered message on the reader
                                    if let Some(message) = ordering.recv(OrderedMessage {
                                        sequence,
                                        payload: frame.payload,
                                    }) {
                                        // A frame being returned means we're up to date
                                        messages.push(channel, message.payload);
                                    } else {
                                        // No frames being returned means we're not up to date
                                        // but we can still check to see if anything has become available
                                        if let Some(drain) = ordering.drain_available() {
                                            for message in drain {
                                                messages.push(channel, message.payload);
                                            }
                                        }
                                    }
                                },

                                // Case 1.2.2: Unordered message
                                false => {
                                    messages.push(channel, frame.payload);
                                },
                            }
                        },
                    }
                },

                // Case 2: Error while reading
                // This doesn't make us terminate
                Some(Err(error)) => {
                    // Record the error and put the peer on 'thinner ice' so to speak
                    // We can't use `melt_ice` as it requires mutable access to the entire type
                    self.ice_thickness = self.ice_thickness.saturating_sub(120);

                    // Trace log for debugging
                    trace!("Error {error:?} while parsing packet"); // TODO: more associated data
                },

                // Case 3: No more packets to read
                // This makes us terminate
                None => {
                    break 'frames;
                },
            }
        }
    }
}