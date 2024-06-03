use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{plugin::PluginConfiguration, prelude::*};
use super::{frames::{frames::{FrameType, RecvFrame}, reader::PacketReaderContext}, Established};

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
                        FrameType::Control => handle_control_frame(
                            frame,
                        ),

                        // Case 1.2: Stardust message frame
                        FrameType::Stardust => handle_stardust_frame(
                            frame,
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
    });
}

fn handle_control_frame(
    frame: RecvFrame,
) {
    todo!()
}

fn handle_stardust_frame(
    frame: RecvFrame,
    channels: &ChannelRegistryInner,
) {
    todo!()
}