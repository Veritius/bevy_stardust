use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{connection::{established::control::ControlFrameIdent, ordering::{OrderedMessage, OrderingManager}}, plugin::PluginConfiguration, prelude::*};
use super::{control::ControlFrame, frames::{frames::{FrameType, RecvFrame}, reader::PacketReaderContext}, Established};

pub(in crate::connection) fn established_reading_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    mut connections: Query<(&mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    connections.par_iter_mut().for_each(|(
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
                            if frame.ident.is_none() {
                                established.ice_thickness = established.ice_thickness.saturating_sub(512);
                                continue
                            }
        
                            let ident = frame.ident.unwrap();
                            if let Ok(ident) = ControlFrameIdent::try_from(ident) {
                                established.control.push(ControlFrame {
                                    ident,
                                    payload: frame.payload,
                                });
                            } else {
                                established.ice_thickness = established.ice_thickness.saturating_sub(512);
                                continue
                            }
                        },

                        // Case 1.2: Stardust message frame
                        FrameType::Stardust => match handle_stardust_frame(
                            frame,
                            &registry,
                            &mut established.orderings,
                            &mut messages,
                        ) {
                            Ok(()) => {},
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
    messages: &mut NetworkMessages<Incoming>,
) -> Result<(), u16> {
    let varint = frame.ident.ok_or(256u16)?;
    let integer: u32 = varint.try_into().map_err(|_| 128u16)?;
    let channel: ChannelId = ChannelId::from(integer);

    let channel_data = registry.channel_config(channel).ok_or(256u16)?;

    // If the message is unordered, push it as follows
    if channel_data.ordered == OrderingGuarantee::Unordered {
        messages.push(channel, frame.payload);
        return Ok(());
    }

    let ordering = orderings.get(channel_data);
    let sequence = frame.order.ok_or(128u16)?;

    let message = ordering.recv(OrderedMessage { sequence, payload: frame.payload });

    if let Some(iter) = ordering.drain_available() {
        for message in iter {
            messages.push(channel, message.payload);
        }
    }

    if let Some(message) = message {
        messages.push(channel, message.payload);
    }

    return Ok(());
}