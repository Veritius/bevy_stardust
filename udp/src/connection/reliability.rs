use std::{cmp::Ordering, collections::BTreeMap, fmt::Debug, time::Instant};
use bytes::{BufMut, Bytes};
use crate::sequences::*;

const BITMASK: u128 = 1;

#[derive(Debug, Clone)]
pub(crate) struct ReliabilityState {
    pub local_sequence: SequenceId,
    pub remote_sequence: SequenceId,
    pub sequence_memory: u128,
}

impl ReliabilityState {
    pub fn new() -> Self {
        Self {
            local_sequence: SequenceId::random(),
            remote_sequence: 0.into(),
            sequence_memory: 0,
        }
    }

    /// Creates a ReliablePacketHeader for your outgoing data.
    pub fn header(&self) -> ReliablePacketHeader {
        ReliablePacketHeader {
            seq: self.local_sequence,
            ack: self.remote_sequence,
            bits: self.sequence_memory
        }
    }

    /// Increments the local sequence by 1
    pub fn advance(&mut self) {
        self.local_sequence += 1;
    }

    /// Acknowledge packets identified in a reliable header. Returns an iterator over the sequences of packets that have been acknowledged by the remote peer.
    pub fn ack(&mut self, header: ReliablePacketHeader, bitfield_bytes: u8) -> impl Iterator<Item = SequenceId> + Clone {
        // Update bitfield and remote sequence
        let diff = header.seq.wrapping_diff(&self.remote_sequence);
        match self.remote_sequence.cmp(&header.seq) {
            Ordering::Greater => {
                // The packet is older, flag it as acknowledged
                self.sequence_memory |= BITMASK.overflowing_shl(diff.into()).0;
            },
            Ordering::Less => {
                // The packet is newer, shift the memory bitfield
                self.remote_sequence = header.seq;
                self.sequence_memory = self.sequence_memory.overflowing_shl(diff.into()).0;
                self.sequence_memory |= BITMASK.overflowing_shl(diff.wrapping_sub(1).into()).0;
            },
            Ordering::Equal => {}, // Shouldn't happen.
        }

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
                    let mask = BITMASK.overflowing_shl(self.cursor.into()).0;
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
            origin: header.ack,
            cursor: 0,
            limit: (bitfield_bytes * 8),
            bitfield: header.bits
        }
    }
}

/// Required information for a reliable packet.
#[derive(Clone, Copy)]
pub struct ReliablePacketHeader {
    pub seq: SequenceId,
    pub ack: SequenceId,
    pub bits: u128,
}

impl ReliablePacketHeader {
    pub fn ser<B: BufMut>(&self, buf: &mut B, bf_size: usize){
        buf.put_u16(self.seq.into());
        buf.put_u16(self.ack.into());
        let bytes = self.bits.to_be_bytes();
        buf.put(&bytes[..bf_size]);
    }
}

impl Debug for ReliablePacketHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReliablePacketHeader")
        .field("seq", &self.seq)
        .field("ack", &self.ack)
        .field("bits", &format_args!("{:b}", self.bits))
        .finish()
    }
}

pub(crate) struct ReliablePackets {
    pub state: ReliabilityState,
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
    pub fn header(&self) -> ReliablePacketHeader {
        self.state.header()
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

    pub fn ack(&mut self, header: ReliablePacketHeader, bitfield_bytes: u8) {
        // Update reliability state
        let iter = self.state.ack(header, bitfield_bytes);

        // Remove all acked packets from storage
        for seq in iter {
            self.unacked.remove(&seq.into());
        }
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
        sequence_memory: 0,
    });

    // This is our other side of the connection.
    let mut bob = ReliablePackets::new(ReliabilityState {
        local_sequence: SequenceId::new(1),
        remote_sequence: SequenceId::new(0),
        sequence_memory: 0,
    });

    // Alice sends a message to Bob
    alice.record(1.into(), empty());
    let alice_header = alice.header();
    assert_eq!(alice_header.seq, 1.into());
    alice.advance();
    assert_eq!(alice.header().seq, 2.into());

    // Bob receives Alice's message
    bob.ack(alice_header, 8);
    assert_eq!(bob.header().ack, 1.into());
    assert_eq!(bob.header().bits, 0b0000_0001);

    // Bob sends a message to Alice
    bob.record(1.into(), empty());
    let bob_header = bob.header();
    assert_eq!(bob_header.seq, 1.into());
    bob.advance();
    assert_eq!(bob.header().seq, 2.into());

    // Alice receives Bob's message
    alice.ack(bob_header, 8);
    assert_eq!(alice.header().ack, 1.into());
    assert_eq!(alice.header().bits, 0b0000_0001);

    // Alice sends a message to Bob
    // Bob does not receive this message
    alice.record(1.into(), empty());
    alice.advance();

    // Alice sends another message to Bob
    alice.record(2.into(), empty());
    let alice_header = alice.header();
    alice.advance();

    // Bob receives Alice's second message
    bob.ack(alice_header, 8);
    assert_eq!(bob.header().ack, 3.into());
    assert_eq!(bob.header().bits, 0b0000_0110);

    // Bob sends a message to Alice
    bob.record(2.into(), empty());
    let bob_header = bob.header();
    assert_eq!(bob.header().seq, 2.into());
    bob.advance();

    // Alice receives Bob's message
    alice.ack(bob_header, 8);
    assert_eq!(alice.header().ack, 2.into());
    assert_eq!(alice.header().bits, 0b0000_0011);

    // Alice should have one packet that needs retransmission
    let mut lost_iter = alice.drain_old(|_| true);
    assert!(lost_iter.next().is_some());
    assert!(lost_iter.next().is_none());
}