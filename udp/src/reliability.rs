use std::{collections::BTreeMap, time::Instant};
use bytes::Bytes;
use untrusted::{Reader, EndOfInput};

struct SentPacket {
    payload: Bytes,
    time: Instant,
}

pub(crate) struct ReliablePacket {
    local_sequence: u16,
    remote_sequence: u16,
    unacked_messages: BTreeMap<u16, SentPacket>,
    packet_memory: u128,
}

impl ReliablePacket {
    /// Gets the header of a reliable packet.
    pub fn get_header(reader: &mut Reader<'_>, bitfield_bytes: usize) -> Result<ReliablePacketHeader, EndOfInput> {
        let sequence = u16::from_be_bytes([
           reader.read_byte()?,
           reader.read_byte()?, 
        ]);

        let ack = u16::from_be_bytes([
            reader.read_byte()?,
            reader.read_byte()?,
        ]);

        let mut ack_bitfield_bytes = [0u8; 16];
        ack_bitfield_bytes[..bitfield_bytes]
            .clone_from_slice(reader.read_bytes(bitfield_bytes)?.as_slice_less_safe());
        let ack_bitfield = u128::from_ne_bytes(ack_bitfield_bytes);

        Ok(ReliablePacketHeader { sequence, ack, ack_bitfield })
    }
}

pub struct ReliablePacketHeader {
    pub sequence: u16,
    pub ack: u16,
    pub ack_bitfield: u128,
}