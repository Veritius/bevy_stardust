use bevy::prelude::*;
use super::{config::ChannelData, registry::ChannelRegistry, id::ChannelId};

/// Panics if a channel component is ever removed, since that should never happen.
pub(in crate) fn panic_on_channel_removal(
    removals: RemovedComponents<ChannelData>,
) {
    if !removals.is_empty() {
        panic!("A channel entity was deleted");
    }
}