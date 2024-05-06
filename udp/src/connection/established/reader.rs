use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::connection::ordering::{OrderedMessage, OrderingManager};
use crate::connection::packets::frames::FrameType;
use crate::connection::packets::reader::{PacketReader, PacketReaderContext};
use crate::connection::reliability::ReliablePackets;
use crate::plugin::PluginConfiguration;
use crate::prelude::*;
use super::Established;

pub(crate) fn established_packet_reader_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    connections.par_iter_mut().for_each(|(entity, mut connection, mut established, mut messages)| {
        // Hack to get around the borrow checker not letting you mutably borrow multiple fields from the same struct at the same time
        #[inline(always)]
        fn split_borrow(established: &mut Established) -> (&mut ReliablePackets, &mut OrderingManager, &mut PacketReader) {
            (&mut established.reliability, &mut established.orderings, &mut established.reader)
        }

        let (reliability, orderings, reader) = split_borrow(&mut established);

        // Context object for the packet reader
        let context = PacketReaderContext {
            queue: &mut connection.recv_queue,
            config: &config,
            reliability,
        };

        // Iterate over all frames
        // This runs until there is no more data to parse
        let mut iter = reader.iter(context);
        'frames: loop {
            match iter.next() {
                // Case 1: Another frame was read
                Some(Ok(frame)) => {
                    match frame.ftype {
                        // Case 1.1: Connection control frame
                        FrameType::Control => {
                            todo!()
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
                                    let ordering = orderings.get(channel_data);

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
                    error!("Error: {error:?}");
                },

                // Case 3: No more packets to read
                // This makes us terminate
                None => {
                    break 'frames;
                },
            }
        }
    });
}