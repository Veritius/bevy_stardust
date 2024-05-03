use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug, time::Instant};
use bytes::Bytes;
use crate::sequences::*;

#[derive(Debug, Clone)]
pub(crate) struct ReliabilityState {
    pub local_sequence: SequenceId,
    pub remote_sequence: SequenceId,
    pub ack_memory: AckMemory,
}

impl ReliabilityState {
    pub fn new() -> Self {
        Self {
            local_sequence: SequenceId::random(),
            remote_sequence: 0.into(),
            ack_memory: AckMemory::default(),
        }
    }

    /// Increments the local sequence by 1
    pub fn advance(&mut self) {
        self.local_sequence += 1;
    }

    /// Acknowledge a remote packet's sequence id.
    pub fn ack_seq(&mut self, seq: SequenceId) {
        // Update bitfield and remote sequence
        let diff = seq.wrapping_diff(&self.remote_sequence);
        match self.remote_sequence.cmp(&seq) {
            Ordering::Greater => {
                // The packet is older, flag it as acknowledged
                self.ack_memory.set_high(diff as u32);
            },
            Ordering::Less => {
                // The packet is newer, shift the memory bitfield
                self.remote_sequence = seq;
                self.ack_memory.shift(diff as u32);
                self.ack_memory.set_high(diff.wrapping_sub(1) as u32);
            },
            Ordering::Equal => {}, // Shouldn't happen.
        }
    }

    /// Record a remote packet's acknowledgements.
    pub fn ack_bits(
        &mut self,
        ack: SequenceId,
        bitfield: AckMemory,
        bitfield_bytes: u8
    ) -> impl Iterator<Item = SequenceId> + Clone {
        // Iterator object for acknowledgements
        #[derive(Clone)]
        struct AcknowledgementIterator {
            origin: SequenceId,
            cursor: u8,
            limit: u8,
            bitfield: u128,
        }

        impl Iterator for AcknowledgementIterator {
            type Item = SequenceId;
        
            fn next(&mut self) -> Option<Self::Item> {
                // Get the ack value
                loop {
                    // Check if we've reached the limit
                    if self.cursor == self.limit { return None }

                    // Get the ack value
                    let mask = AckMemory::BITMASK.overflowing_shl(self.cursor.into()).0;
                    if self.bitfield & mask == 0 { self.cursor += 1; continue }
                    let ack = self.origin + self.cursor as u16;

                    // Success, advance cursor and return
                    self.cursor += 1;
                    return Some(ack.into())
                }
            }
        }

        // Create iterator
        AcknowledgementIterator {
            origin: ack,
            cursor: 0,
            limit: (bitfield_bytes * 8),
            bitfield: bitfield.0,
        }
    }
}

pub(crate) struct ReliablePackets {
    state: ReliabilityState,
    unacked: BTreeMap<u16, UnackedPacket>,
}

impl ReliablePackets {
    pub fn new(state: ReliabilityState) -> Self {
        Self {
            unacked: BTreeMap::default(),
            state,
        }
    }

    #[inline]
    pub fn advance(&mut self) {
        self.state.advance()
    }

    pub fn record(&mut self, sequence: SequenceId, payload: Bytes) {
        self.unacked.insert(sequence.into(), UnackedPacket {
            payload,
            time: Instant::now(),
        });
    }

    pub fn clone_state(&self) -> ReliabilityState {
        self.state.clone()
    }

    #[inline]
    pub fn ack_seq(&mut self, seq: SequenceId) {
        self.state.ack_seq(seq)
    }

    pub fn rec_ack(
        &mut self,
        ack: SequenceId,
        bitfield: AckMemory,
        bitfield_bytes: u8,
    ) {
        let iter = self.state.ack_bits(ack, bitfield, bitfield_bytes);
        for seq in iter {
            self.unacked.remove(&seq.into());
        }
    }

    fn ack_state_testing_only(&mut self, state: ReliabilityState) {
        self.ack_seq(state.local_sequence);
        self.rec_ack(state.remote_sequence, state.ack_memory, 16);
    }

    pub fn drain_old<'a, Filter: Fn(Instant) -> bool + 'a>(&'a mut self, filter: Filter) -> impl Iterator<Item = UnackedPacket> + 'a {
        // TODO: When btree_extract_if is stabilised, use that instead.
        struct FilterTaker<'a, Filter>(&'a mut ReliablePackets, Filter);
        impl<'a, Filter: Fn(Instant) -> bool> Iterator for FilterTaker<'a, Filter> {
            type Item = UnackedPacket;
        
            fn next(&mut self) -> Option<Self::Item> {
                // Try to find a key
                let key = self.0.unacked.iter()
                    .filter(|(_, v)| { (self.1)(v.time) })
                    .map(|(k, _)| *k)
                    .next()?;
                
                // Take the packet from the map and return it
                return self.0.unacked.remove(&key);
            }
        }

        // Return the iterator
        return FilterTaker(self, filter);
    }
}

#[derive(Default, Clone)]
pub struct AckMemory(u128);

impl AckMemory {
    const BITMASK: u128 = 1;

    #[inline]
    pub fn from_array(array: [u8; 16]) -> Self {
        Self(u128::from_be_bytes(array))
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, ()> {
        let len = slice.len().max(16);
        if len == 0 { return Err(()); }
        let mut bytes = [0u8; 16];
        bytes[..len].copy_from_slice(&slice[..len]);
        return Ok(Self::from_array(bytes))
    }

    #[inline]
    pub fn into_array(&self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    #[inline]
    pub fn shift(&mut self, bits: u32) {
        self.0 = self.0.overflowing_shl(bits).0
    }

    #[inline]
    pub fn set_high(&mut self, idx: u32) {
        self.0 |= Self::BITMASK.overflowing_shl(idx).0;
    }
}

impl Debug for AckMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:b}", self.0))
    }
}

#[derive(Debug)]
pub(crate) struct UnackedPacket {
    pub payload: Bytes,
    time: Instant,
}

#[test]
fn conversation_test() {
    static EMPTY: &[u8] = &[];

    // An empty Bytes object to test with.
    #[inline]
    fn empty() -> Bytes {
        Bytes::from_static(EMPTY)
    }

    // We can't use ReliabilityState::new() since it generates random values.
    // This is our first side of the connection.
    let mut alice = ReliablePackets::new(ReliabilityState {
        local_sequence: SequenceId::new(1),
        remote_sequence: SequenceId::new(0),
        ack_memory: AckMemory::default(),
    });

    // This is our other side of the connection.
    let mut bob = ReliablePackets::new(ReliabilityState {
        local_sequence: SequenceId::new(1),
        remote_sequence: SequenceId::new(0),
        ack_memory: AckMemory::default(),
    });

    // Alice sends a message to Bob
    alice.record(1.into(), empty());
    let alice_header = alice.clone_state();
    assert_eq!(alice_header.local_sequence, 1.into());
    alice.advance();
    assert_eq!(alice.clone_state().local_sequence, 2.into());

    // Bob receives Alice's message
    bob.ack_state_testing_only(alice_header);
    assert_eq!(bob.clone_state().remote_sequence, 1.into());
    assert_eq!(bob.clone_state().ack_memory.0, 0b0000_0001);

    // Bob sends a message to Alice
    bob.record(1.into(), empty());
    let bob_header = bob.clone_state();
    assert_eq!(bob_header.local_sequence, 1.into());
    bob.advance();
    assert_eq!(bob.clone_state().local_sequence, 2.into());

    // Alice receives Bob's message
    alice.ack_state_testing_only(bob_header);
    assert_eq!(alice.clone_state().remote_sequence, 1.into());
    assert_eq!(alice.clone_state().ack_memory.0, 0b0000_0001);

    // Alice sends a message to Bob
    // Bob does not receive this message
    alice.record(1.into(), empty());
    alice.advance();

    // Alice sends another message to Bob
    alice.record(2.into(), empty());
    let alice_header = alice.clone_state();
    alice.advance();

    // Bob receives Alice's second message
    bob.ack_state_testing_only(alice_header);
    assert_eq!(bob.clone_state().remote_sequence, 3.into());
    assert_eq!(bob.clone_state().ack_memory.0, 0b0000_0110);

    // Bob sends a message to Alice
    bob.record(2.into(), empty());
    let bob_header = bob.clone_state();
    assert_eq!(bob.clone_state().local_sequence, 2.into());
    bob.advance();

    // Alice receives Bob's message
    alice.ack_state_testing_only(bob_header);
    assert_eq!(alice.clone_state().remote_sequence, 2.into());
    assert_eq!(alice.clone_state().ack_memory.0, 0b0000_0011);

    // Alice should have one packet that needs retransmission
    let mut lost_iter = alice.drain_old(|_| true);
    assert!(lost_iter.next().is_some());
    assert!(lost_iter.next().is_none());
}