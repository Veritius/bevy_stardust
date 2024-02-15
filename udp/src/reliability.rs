use std::{collections::BTreeMap, time::Instant};
use bytes::Bytes;

struct SentPacket {
    payload: Bytes,
    time: Instant,
}

pub(crate) struct ReliableMessages {
    local_sequence: u16,
    remote_sequence: u16,
    unacked_messages: BTreeMap<u16, SentPacket>,
    packet_memory: u128,
}