use bevy::prelude::*;
use super::{components::ChannelData, registry::ChannelRegistry, id::ChannelId};

/// Panics if a channel component is ever removed, since that should never happen.
pub(in crate::shared) fn panic_on_channel_removal(
    removals: RemovedComponents<ChannelData>,
) {
    if !removals.is_empty() {
        panic!("A channel entity was deleted");
    }
}

pub(in crate::shared) fn clear_octet_buffers_system(
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