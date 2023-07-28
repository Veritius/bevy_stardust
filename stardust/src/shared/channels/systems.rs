use bevy::prelude::RemovedComponents;
use super::components::ChannelData;

/// Panics if a channel component is ever removed, since that should never happen.
pub(in crate::shared) fn panic_on_channel_removal(
    removals: RemovedComponents<ChannelData>,
) {
    if !removals.is_empty() {
        panic!("A channel entity was deleted");
    }
}