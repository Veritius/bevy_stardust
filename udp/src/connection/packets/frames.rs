use bevy_stardust::channels::ChannelId;
use bytes::Bytes;

pub(in crate::connection) enum Frame {
    Control(ControlFrame),
    Stardust(StardustFrame),
}

pub(in crate::connection) struct ControlFrame {
    pub payload: Bytes,
}

pub(in crate::connection) struct StardustFrame {
    pub channel: ChannelId,
    pub payload: Bytes,
}