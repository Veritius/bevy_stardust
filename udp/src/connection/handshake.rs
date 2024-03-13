//! Handshake implementation. Based on TCP and QUIC.
//! This protocol is largely distinct from the rest of the code.

/*

This is an explanation of how the standard handshake works.
For the purposes of the explanation, we will use the following terms.
- "Initiator" to refer to the peer making the outgoing connection, and acting like a client.
- "Listener" to refer to the peer receiving the incoming connection, and acting like a server.

While the handshake is not set in stone, a few parts of it are.
For the first packet, the first four fields will always remain the same.
For the second packet, the first five fields will always remain the same.
This is to ensure good errors when trying to connect to an old version of a game.

After a response code, the layout of the packet may be different, based on the code.
Different response codes will have different layouts, which are listed below.

The first packet is sent by the Initiator to the Listener, to start a connection.
It consists of the following information, with square brackets showing the type.
[u64] Transport identifier
[u32] Transport version (minor)
[u32] Transport version (major)
[u64] Application identifier
[u16] Sequence identifier
(total length 26 bytes)

The second packet is sent by the Listener to the Initiator, to acknowledge the connection response.
This establishes the acknowledgement state of the Initiator, and begins the same process for the Listener.
[u64] Transport identifier
[u32] Transport version (minor)
[u32] Transport version (major)
[u64] Application identifier
[u16] Response code

0 (Continue)
===============================
[u16] Sequence identifier
[u16] Acknowledgement
[u16] Ack bitfield
(total length 32 bytes)

The third packet is sent by the Initiator to the Listener, to acknowledge the Initiator's sequence id.
This establishes the acknowledgement state of the Listener. At this point, both peers start communicating reliably.
[u16] Response code

0 (Continue)
=========================
[u16] Sequence identifier
[u16] Acknowledgement
[u16] Ack bitfield
(total length 8 bytes)

*/

use bytes::{BufMut, Bytes, BytesMut};
use crate::packet::PacketQueue;
use super::{reliability::ReliabilityData, statemachine::PotentialStateTransition, timing::ConnectionTimings};

/// Handshake state machine for connections.
#[derive(Debug)]
pub(super) struct ConnectionHandshake {
    context: HandshakeContext,
    state: HandshakeState,
    reliability: ReliabilityData,
}

#[derive(Debug)]
pub(super) struct HandshakeContext {
    pub transport_identifier: u64,
    pub transport_version_minor: u32,
    pub transport_version_major: u32,
    pub application_identifier: u64,
}

#[derive(Debug)]
enum HandshakeState {
    RelSynSent,
    RelSynRecv,
    Finished,
    Failure,
}

impl ConnectionHandshake {
    pub fn new_incoming(
        context: HandshakeContext,
    ) -> Self {
        Self {
            context,
            state: HandshakeState::RelSynRecv,
            reliability: ReliabilityData::new(),
        }
    }

    pub fn new_outgoing(
        context: HandshakeContext,
    ) -> Self {
        Self {
            context,
            state: HandshakeState::RelSynSent,
            reliability: ReliabilityData::new(),
        }
    }

    pub fn poll(
        &mut self,
        timings: &mut ConnectionTimings,
        packets: &mut PacketQueue,
    ) -> PotentialStateTransition<Self, ()> {
        match self.state {
            HandshakeState::RelSynSent => {
                if let Some(packet) = packets.pop_incoming() {
                    todo!()
                }

                todo!()
            },
            HandshakeState::RelSynRecv => todo!(),
            HandshakeState::Finished => todo!(),
            HandshakeState::Failure => todo!(),
        }
    }
}

fn build_first_pkt(
    context: &HandshakeContext,
    reliability: &ReliabilityData,
) -> Bytes {
    // Create buffer for storing bytes
    let mut buf = BytesMut::with_capacity(26);

    // Info about our app
    buf.put_u64(context.transport_identifier);
    buf.put_u32(context.transport_version_major);
    buf.put_u32(context.transport_version_minor);
    buf.put_u64(context.application_identifier);

    // Info about reliability
    buf.put_u16(reliability.local_sequence);

    // Make immutable and return
    return buf.freeze();
}

fn build_second_pkt_ok(
    context: &HandshakeContext,
    reliability: &ReliabilityData,
) -> Bytes {
    // Create buffer for storing bytes
    let mut buf = BytesMut::with_capacity(32);

    // Info about our app
    buf.put_u64(context.transport_identifier);
    buf.put_u32(context.transport_version_major);
    buf.put_u32(context.transport_version_minor);
    buf.put_u64(context.application_identifier);

    // Response code
    buf.put_u16(HandshakeResponseCode::Continue as u16);
    
    // Info about reliability
    buf.put_u16(reliability.local_sequence);
    buf.put_u16(reliability.remote_sequence);

    // Insert the firstmost bytes from the bitfield
    // This is done to save space, since the rest of it doesn't matter
    let bitfield = reliability.sequence_memory.to_be_bytes();
    buf.put_u8(bitfield[0]);
    buf.put_u8(bitfield[1]);

    // Make immutable and return
    return buf.freeze();
}

fn build_third_pkt_ok(
    context: &HandshakeContext,
    reliability: &ReliabilityData,
) -> Bytes {
    // Create buffer for storing bytes
    let mut buf = BytesMut::with_capacity(8);

    // Response code
    buf.put_u16(HandshakeResponseCode::Continue as u16);

    // Info about reliability
    buf.put_u16(reliability.local_sequence);
    buf.put_u16(reliability.remote_sequence);

    // Again, insert the firstmost bytes from the bitfield
    let bitfield = reliability.sequence_memory.to_be_bytes();
    buf.put_u8(bitfield[0]);
    buf.put_u8(bitfield[1]);

    // Make immutable and return
    return buf.freeze();
}

#[repr(u16)]
enum HandshakeResponseCode {
    Continue = 0,
}

impl TryFrom<u16> for HandshakeResponseCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Continue,
            _ => { return Err(()) }
        })
    }
}