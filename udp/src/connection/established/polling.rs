use bevy::prelude::*;
use bevy_stardust::prelude::*;
use smallvec::SmallVec;
use crate::{connection::ordering::OrderingManager, plugin::PluginConfiguration, prelude::*};
use super::{control::{ControlFrame, ControlFrameIdent}, frames::{frames::{FrameType, RecvFrame}, reader::PacketReaderContext}, Established};

pub(in crate::connection) fn established_reading_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    connections.par_iter_mut().for_each(|(
        entity,
        mut connection,
        mut established,
        mut messages,
    )| {
        // Some checks to avoid unnecessary work
        if connection.recv_queue.is_empty() { return }

        // Reborrows to please the borrow checker
        let connection = &mut *connection;
        let established = &mut *established;

        // Context object for the packet reader
        let context = PacketReaderContext {
            queue: &mut connection.recv_queue,
            config: &config,
            reliability: &mut established.reliability,
        };

        // Iterate over all frames
        // This runs until there is no more data to parse
        let mut iter = established.reader.iter(context);
        'frames: loop {
            match iter.next() {
                // Case 1: Another frame was read
                Some(Ok(frame)) => {
                    match frame.ftype {
                        // Case 1.1: Connection control frame
                        FrameType::Control => {
                            // All control frames have an identifying value
                            if frame.ident.is_none() {
                                established.ice_thickness = established.ice_thickness.saturating_sub(512);
                                continue;
                            }

                            // Push the control frame to the queue
                            let ident = frame.ident.unwrap();
                            established.cframes.push(ControlFrame {
                                payload: frame.payload,
                                ident: match ControlFrameIdent::try_from(ident) {
                                    Ok(f) => f,
                                    Err(_) => {
                                        // Control frame had an invalid ident
                                        established.ice_thickness = established.ice_thickness.saturating_sub(512);
                                        continue;
                                    },
                                },
                            })
                        },

                        // Case 1.2: Stardust message frame
                        FrameType::Stardust => match handle_stardust_frame(
                            frame,
                            &registry,
                            &mut established.orderings,
                        ) {
                            Ok((channel, payload)) => messages.push(channel, payload),

                            Err(amt) => {
                                established.ice_thickness = established.ice_thickness.saturating_sub(amt);
                            },
                        },
                    }
                },

                // Case 2: Error while reading
                // This doesn't make us terminate
                Some(Err(error)) => {
                    // Record the error and put the peer on 'thinner ice' so to speak
                    established.ice_thickness = established.ice_thickness.saturating_sub(120);

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
    });
}

fn handle_stardust_frame(
    frame: RecvFrame,
    registry: &ChannelRegistryInner,
    orderings: &mut OrderingManager,
) -> Result<(ChannelId, Bytes), u16> {
    let varint = frame.ident.ok_or(256u16)?;
    let integer: u32 = varint.try_into().map_err(|_| 128u16)?;
    let channel: ChannelId = ChannelId::from(integer);

    let channel_data = registry.channel_config(channel).ok_or(256u16)?;

    // TODO: Handle orderings

    return Ok((channel, frame.payload));
}