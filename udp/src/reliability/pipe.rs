use std::collections::BTreeMap;
use bytes::Bytes;
use super::sequence_greater_than;

pub(super) struct ReliablePipe {
    /// The sequence ID we're using to send messages.
    local_sequence: u16,
    /// The highest sequence id we've heard from our friend.
    remote_sequence: u16,
    /// Storage for messages we've sent that haven't been acknowledged yet.
    unacked_messages: BTreeMap<u16, Bytes>,
    /// Messages we've received from our friend over the internet
    /// This is a u128 so we have a lot of space to work with
    /// and for if the user wants to have a longer ack bitfield range
    received_packets: u128,
}

impl ReliablePipe {
    /// Creates a new `ReliablePipe` with a sequence ID.
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
    pub fn send(&mut self, scratch: &mut [u8], payload: Bytes) -> usize {
        // Append to the unacknowledged messages map
        let seq = self.local_sequence.clone();
        self.local_sequence = self.local_sequence.wrapping_add(1);
        self.unacked_messages.insert(seq, payload.clone());

        // Create the 'scratch' buffer
        let length = 8 + &payload.len();
        scratch[0..2].clone_from_slice(&self.local_sequence.to_be_bytes());
        scratch[2..4].clone_from_slice(&self.remote_sequence.to_be_bytes());
        scratch[4..8].clone_from_slice(&self.received_packets.to_be_bytes());
        scratch[8..length].clone_from_slice(&payload);

        // Return bytes written
        return length
    }

    /// "Receives" the contents of a reliable packet, removing the header and returning a slice containing the payload.
    pub fn receive<'a>(&mut self, buffer: &'a [u8]) -> &'a [u8] {
        // Create some variables
        let their_remote = u16::from_be_bytes(buffer[0..2].try_into().unwrap());
        let their_ack = u16::from_be_bytes(buffer[2..4].try_into().unwrap());
        let their_bitfield = u32::from_be_bytes(buffer[4..8].try_into().unwrap());
        let their_payload = &buffer[8..];

        // Update the remote sequence
        if sequence_greater_than(their_remote, self.remote_sequence) {
            self.remote_sequence = their_remote;
        }

        // Flag their message as received
        todo!();

        // Acknowledge the ack sequence packet
        self.unacked_messages.remove(&their_ack);

        // Acknowledge all sequences in the bitfield
        todo!();

        // Return the payload
        their_payload
    }
}