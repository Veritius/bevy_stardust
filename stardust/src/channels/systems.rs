use bevy::prelude::*;
use super::{config::ChannelData, registry::ChannelRegistry, id::ChannelId, incoming::IncomingNetworkMessages};

/// Panics if a channel component is ever removed, since that should never happen.
pub(in crate) fn panic_on_channel_removal(
    removals: RemovedComponents<ChannelData>,
) {
    if !removals.is_empty() {
        panic!("A channel entity was deleted");
    }
}

pub(in crate) fn clear_outgoing_buffers_system(
    registry: Res<ChannelRegistry>,
) {
    let count = registry.channel_count();
    for index in 0..count {
        let channel = ChannelId::try_from(index).unwrap();
        registry
            .get_outgoing_arc(channel)
            .unwrap()
            .write()
            .unwrap()
            .clear();
    }
}

pub(in crate) fn clear_incoming_buffers_system(
    mut query: Query<&mut IncomingNetworkMessages>,
) {
    for mut outgoing in query.iter_mut() {
        outgoing.clear();
    }
}