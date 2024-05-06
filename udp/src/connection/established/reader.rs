use bevy::prelude::*;
use bevy_stardust::prelude::*;
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
        fn split_borrow(established: &mut Established) -> (&mut ReliablePackets, &mut PacketReader) {
            (&mut established.reliability, &mut established.reader)
        }

        let (reliability, reader) = split_borrow(&mut established);

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
                            if !registry.channel_exists(channel) {
                                todo!()
                            }

                            // TODO: Ordering stuff

                            // Add to the incoming queue component
                            messages.push(channel, frame.payload);
                        },
                    }
                },

                // Case 2: Error while reading
                // This doesn't make us terminate
                Some(Err(error)) => {
                    todo!()
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