use std::time::Instant;

use bevy_stardust::channels::ChannelId;
use bytes::Bytes;

pub(in crate::connection) struct Frame {
    pub priority: u32,
    pub instant: Instant,
    pub inner: FrameInner,
}

pub(in crate::connection) enum FrameInner {
    Control(ControlFrame),
    Handshake(HandshakeFrame),
    Stardust(StardustFrame),
}

pub(in crate::connection) struct ControlFrame {
    pub payload: Bytes,
}

pub(in crate::connection) struct HandshakeFrame {
    pub payload: Bytes,
}

pub(in crate::connection) struct StardustFrame {
    pub channel: ChannelId,
    pub payload: Bytes,
}