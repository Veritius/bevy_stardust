use std::{collections::BTreeMap, time::{Instant, Duration}};
use bytes::Bytes;
use crate::config::PluginConfig;

use super::{sequence_greater_than, SentPacket};

const DROPPED_TIMEOUT: Duration = Duration::from_millis(1000);
const BITMASK: u128 = 1 << 127;

pub(super) struct ReliableRiver {
    /// The sequence ID we're using to send messages.
    /// Used when we send messages
    local_sequence: u16,

    /// Storage for messages we've sent that they haven't acknowledged yet.
    unacked_messages: BTreeMap<u16, SentPacket>,

    /// The highest sequence id we've heard from our friend.
    /// Used with received_packets to track what we've received
    remote_sequence: u16,

    /// Messages they've sent that we have/haven't acknowledged yet, relative to remote_sequence
    /// This is a u128 so we have a lot of space to work with, and for if the user wants to have a longer ack bitfield range
    received_packets: u128,
}

impl ReliableRiver {
    /// Creates a new `ReliableRiver` with a sequence ID.
    pub fn new(local: u16) -> Self {
        Self {
            local_sequence: local,
            remote_sequence: 0,
            unacked_messages: BTreeMap::new(),
            received_packets: 0,
        }
    }

    /// Sets the internal remote sequence ID to `remote`.
    /// 
    /// **ONLY USE THIS DURING A HANDSHAKE. THIS WILL BREAK RELIABILITY OTHERWISE!**
    pub fn set_remote(&mut self, remote: u16) {
        self.remote_sequence = remote;
    }

    /// "Sends" a payload, storing it for potential resending,
    /// and writing the reliable header and `payload` to `scratch`.
    /// 
    /// Panics if `scratch` is too short. It must be at least 8 + `payload`'s length.
    pub fn send(&mut self, config: &PluginConfig, scratch: &mut [u8], payload: Bytes) -> usize {
        // Some values we use later
        let bitfield_idx = config.bitfield_bytes as usize + 4;

        // Append to the unacknowledged messages map
        let seq = self.local_sequence.clone();
        self.local_sequence = self.local_sequence.wrapping_add(1);
        self.unacked_messages.insert(seq, SentPacket {
            data: payload.clone(),
            time: Instant::now()
        });

        // Create the 'scratch' buffer
        scratch[0..2].clone_from_slice(&self.local_sequence.to_be_bytes());
        scratch[2..4].clone_from_slice(&self.remote_sequence.to_be_bytes());
        scratch[4..bitfield_idx].clone_from_slice(&self.received_packets.to_be_bytes()[0..4]);
        scratch[bitfield_idx..bitfield_idx+payload.len()].clone_from_slice(&payload);

        // Return bytes written
        return bitfield_idx+payload.len()
    }

    /// "Receives" the contents of a reliable packet, removing the header and returning a slice containing the payload.
    pub fn receive<'a>(&mut self, config: &PluginConfig, buffer: &'a [u8]) -> &'a [u8] {
        // Sequence values
        let their_remote = u16::from_be_bytes(buffer[0..2].try_into().unwrap());
        let their_ack = u16::from_be_bytes(buffer[2..4].try_into().unwrap());

        // Create the bitfield var
        let bytes_usize = config.bitfield_bytes as usize;
        let mut field_bytes = [0u8; 16];
        field_bytes[..(config.bitfield_bytes as usize)]
            .clone_from_slice(&buffer[4..4+bytes_usize]);
        let their_bitfield = u128::from_ne_bytes(field_bytes);
        let bit_len = bytes_usize as usize * 8;

        // Payload slice for returning it later
        let their_payload = &buffer[4+bytes_usize..];

        // Update the remote sequence
        let seq_diff = super::wrapping_diff(their_remote, self.remote_sequence);
        if sequence_greater_than(their_remote, self.remote_sequence) {
            // Shift the bitfield to account for the new maximum
            self.remote_sequence = their_remote;
            self.received_packets >>= seq_diff;
        } else {
            // Flag the packet as registered
            self.received_packets |= BITMASK >> seq_diff;
        }

        // Acknowledge the ack sequence packet
        self.unacked_messages.remove(&their_ack);

        // Acknowledge all sequences in the bitfield
        for idx in 0..bit_len {
            let mask = BITMASK >> idx;
            if their_bitfield & mask == 0 { continue }
            let ack = their_ack.wrapping_sub(idx as u16);
            self.unacked_messages.remove(&ack);
        }

        // Return the payload
        their_payload
    }

    /// Returns an iterator over all packets that need sending.
    pub fn timed_out(&self) -> impl Iterator<Item = (u16, &Bytes)> {
        let now = Instant::now();
        self.unacked_messages.iter()
            .filter(move |(k, v)| {
                v.time.duration_since(now) > DROPPED_TIMEOUT
            })
            .map(|(k, v)| (*k, &v.data))
    }
}

#[allow(unused_imports)]
mod tests {
    use bytes::Bytes;
    use crate::config::PluginConfig;
    use super::ReliableRiver;

    const PLUGIN_CONFIG: PluginConfig = PluginConfig {
        river_count: 8,
        bitfield_bytes: 4,
    };

    #[test]
    fn simple_reliable_no_drop() {
        let mut scratch = [0u8; 256];

        let mut one = ReliableRiver::new(0);
        let mut two = ReliableRiver::new(0);

        let len = one.send(&PLUGIN_CONFIG, &mut scratch, Bytes::from("hello"));
        assert_eq!(one.unacked_messages.get(&0).unwrap().data.as_ref(), b"hello");
        let pld = two.receive(&PLUGIN_CONFIG, &scratch[..len]);
        assert_eq!(pld, b"hello");

        let len = two.send(&PLUGIN_CONFIG, &mut scratch, Bytes::from("world"));
        assert_eq!(two.unacked_messages.get(&0).unwrap().data.as_ref(), b"world");
        let pld = one.receive(&PLUGIN_CONFIG, &scratch[..len]);
        assert_eq!(pld, b"world");
    }
}