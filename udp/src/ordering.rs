use std::collections::BTreeMap;
use bevy_stardust::prelude::*;

pub(crate) struct OrderingData {
    pub control: ChannelOrderingData,
    pub channels: BTreeMap<ChannelId, ChannelOrderingData>,
}

pub(crate) struct ChannelOrderingData {

}