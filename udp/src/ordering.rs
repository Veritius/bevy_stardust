use bevy_stardust::prelude::*;

pub(crate) struct OrderingData {
    pub control: ChannelOrderingData,
    pub channels: Vec<ChannelOrderingData>,
}

pub(crate) struct ChannelOrderingData {
    reliable: bool,
    queue: Vec<(u16, OctetString)>,
}