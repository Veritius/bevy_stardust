use bevy_stardust::prelude::*;
use bytes::Bytes;
use crate::plugin::PluginConfiguration;
use super::{ordering::OrderingManager, packets::frames::RecvFrame};

pub(super) fn read_stardust_frame(
    frame: RecvFrame,
    config: &PluginConfiguration,
    channels: &ChannelRegistryInner,
    orderings: &mut OrderingManager,
) -> Result<(ChannelId, Bytes), ()> {
    todo!()
}