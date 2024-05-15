use bevy_stardust::prelude::*;
use bytes::Bytes;
use super::{ordering::OrderingManager, packets::frames::RecvFrame};

pub(super) fn read_stardust_frame(
    frame: RecvFrame,
    channels: &ChannelRegistryInner,
    orderings: &mut OrderingManager,
) -> Result<(ChannelId, Bytes), ()> {
    todo!()
}