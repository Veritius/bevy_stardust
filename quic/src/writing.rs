use bevy_ecs::prelude::*;
use bevy_stardust::{connections::groups::NetworkGroup, prelude::*};
use crate::{streams::OutgoingStreamData, QuicConnection};

pub(super) fn write_messages_to_streams_system(
    network_groups: Query<&NetworkGroup>,
    mut connections: Query<&mut QuicConnection, With<NetworkPeer>>,
    registry: Res<ChannelRegistry>,
    writer: NetworkOutgoingReader,
) {
    // This could maybe be made to run in parallel
    for (channel, target, bytes) in writer.iter_all() {
        if let Ok(group) = network_groups.get(target) {
            // If a group is set as a target, we send the data to each connection in the group
            for target in group.0.iter() {
                if let Ok(mut connection) = connections.get_mut(*target) {
                    write_message_to_connection(
                        &registry,
                        channel,
                        &mut connection,
                        bytes
                    );
                }
            }
        } else {
            // Otherwise we just go straight to the connections query
            if let Ok(mut connection) = connections.get_mut(target) {
                write_message_to_connection(
                    &registry,
                    channel,
                    &mut connection,
                    bytes
                );
            }
        }
    }
}

fn write_message_to_connection(
    registry: &ChannelRegistry,
    channel: ChannelId,
    connection: &mut QuicConnection,
    bytes: &Bytes,
) {
    let bytes_for_channel = crate::misc::bytes_for_channel_id(registry.channel_count()) as usize;
    let channel_bytes = Into::<[u8;4]>::into(channel);

    // TODO: This repeatedly iterates streams that we can remember to be blocked.
    // Quinn provides events to notify us of unblocked channels, so we can use that.
    let data = registry.get_from_id(channel).unwrap();
    match (data.reliable, data.ordered) {
        (ReliabilityGuarantee::Unreliable, OrderingGuarantee::Unordered) => todo!(),

        (ReliabilityGuarantee::Unreliable, _) => todo!(),

        // Reliable message that doesn't need ordering
        // This is put on a "transient" stream in a queue
        (ReliabilityGuarantee::Reliable, OrderingGuarantee::Unordered) => {
            // Create new stream data buffering object
            let sid = connection.inner.get_mut().streams().open(quinn_proto::Dir::Uni).unwrap();
            let mut send_stream = connection.inner.get_mut().send_stream(sid);
            let mut stream_data = OutgoingStreamData::new(sid, &channel_bytes[..bytes_for_channel]);

            // Queue bytes to write and append it to the queue if it doesn't finish immediately
            stream_data.push(&bytes);
            stream_data.try_write(&mut send_stream).unwrap(); // TODO: Handle without panic
            if !stream_data.is_drained() { connection.transient_send_streams.push_back(stream_data); }
        },

        // Reliable message that needs ordering
        // This is put on a "persistent" stream in a map
        (ReliabilityGuarantee::Reliable, _) => {
            // Get or create stream data buffering object
            let stream_data = connection
            .persistent_send_streams
            .entry(channel)
            .or_insert_with(|| {
                let c_inner = connection.inner.get_mut();
                let sid = c_inner.streams().open(quinn_proto::Dir::Uni).unwrap();
                OutgoingStreamData::new(sid, &channel_bytes[..bytes_for_channel])
            });

            // Queue bytes then try to send some of it
            let mut send_stream = connection.inner.get_mut().send_stream(stream_data.id);
            stream_data.push(&bytes);
            stream_data.try_write(&mut send_stream).unwrap(); // TODO: Handle without panic
        },
    }
}