pub(super) mod builder;
pub(super) mod frames;
pub(super) mod reader;

use std::time::Instant;
use bytes::Bytes;

pub(crate) struct RecvPacket {
    pub time: Instant,
    pub payload: Bytes,
}

pub(crate) struct SendPacket {
    pub payload: Bytes,
}