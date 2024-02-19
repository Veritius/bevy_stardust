use bevy_ecs::prelude::*;
use bevy_stardust::{connections::groups::NetworkGroup, prelude::*};
use crate::QuicConnection;

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
    todo!()
}