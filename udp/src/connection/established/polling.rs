use std::{collections::VecDeque, mem::swap};

use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{connection::reliability::{ReliabilityState, ReliablePackets}, plugin::PluginConfiguration, prelude::*};
use super::{frames::{frames::{FrameType, RecvFrame}, reader::{PacketParser, PacketReaderContext}}, Established};

/// Runs [`poll`](Established::poll) on all [`Established`] entities.
pub(crate) fn established_polling_system(
    registry: Res<ChannelRegistry>,
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    connections.par_iter_mut().for_each(|(
        entity,
        mut connection,
        mut established,
        mut messages
    )| {
        // Some checks to avoid unnecessary work
        if connection.recv_queue.is_empty() { return }

        // Local variables that store fields we swap out
        let mut recv_queue = VecDeque::new();
        let mut reliability = ReliablePackets::new(ReliabilityState::new());
        let mut reader = PacketParser::default();

        // Macro to ensure we swap everything back exactly the same
        macro_rules! do_swap {
            () => {
                swap(&mut connection.recv_queue, &mut recv_queue);
                swap(&mut established.reliability, &mut reliability);
                swap(&mut established.reader, &mut reader);
            };
        }

        // Swaps to allow us to mutably borrow fields
        do_swap!();

        // Context object for the packet reader
        let context = PacketReaderContext {
            queue: &mut recv_queue,
            config: &config,
            reliability: &mut reliability,
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
                        FrameType::Control => handle_control_frame(
                            frame,
                            &mut connection,
                            &mut established,
                        ),

                        // Case 1.2: Stardust message frame
                        FrameType::Stardust => handle_stardust_frame(
                            frame,
                            &mut connection,
                            &mut established,
                            &registry,
                        ),
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

        // Swap everything back
        drop(iter);
        do_swap!();
    });
}

fn handle_control_frame(
    frame: RecvFrame,
    connection: &mut Connection,
    established: &mut Established,
) {
    todo!()
}

fn handle_stardust_frame(
    frame: RecvFrame,
    connection: &mut Connection,
    established: &mut Established,
    channels: &ChannelRegistryInner,
) {
    todo!()
}