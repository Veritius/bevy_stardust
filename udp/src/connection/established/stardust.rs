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
    // Get the channel ID from the frame
    let frame_ident: u64 = frame.ident.ok_or(())?.into();
    if frame_ident > 2u64.pow(32) { return Err(()) }
    let channel = ChannelId::from(frame_ident as u32);

    // Get the channel data from the id
    let channel_data = channels.channel_config(channel).ok_or(())?;
    // let ordered = channel_data.ordered != OrderingGuarantee::Unordered;

    match channel_data.ordered {
        OrderingGuarantee::Unordered => { return Ok((channel, frame.payload)) },
        _ => todo!()
    }
}