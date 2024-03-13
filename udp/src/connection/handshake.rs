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
[u16] Response code
[u64] Transport identifier
[u32] Transport version (minor)
[u32] Transport version (major)
[u64] Application identifier

0 (Accept)
===============================
[u16] Sequence identifier
[u16] Acknowledgement
[u16] Ack bitfield
(total length 32 bytes)

The third packet is sent by the Initiator to the Listener, to acknowledge the Initiator's sequence id.
This establishes the acknowledgement state of the Listener. At this point, both peers start communicating reliably.
[u16] Response code

0 (Accept)
=========================
[u16] Sequence identifier
[u16] Acknowledgement
[u16] Ack bitfield
(total length 8 bytes)

The following response codes have identical additional data after them, and don't need to be repeated.
2 (RejectHumanReason)
=========================
[u8;n] UTF-8 string data

The following response codes do not have any additional data and so do not appear in the set.
1 (RejectNoReason)
3 (RejectBadPacket)

*/

use std::time::{Duration, Instant};
use bytes::{BufMut, Bytes, BytesMut};
use untrusted::{EndOfInput, Input, Reader};
use bevy_ecs::prelude::Resource;
use crate::{packet::{OutgoingPacket, PacketQueue}, utils::slice_to_array};
use super::{reliability::ReliabilityData, statemachine::PotentialStateTransition, timing::{timeout_check, ConnectionTimings}};

const HANDSHAKE_RESEND_DURATION: Duration = Duration::from_secs(5);

/// Handshake state machine for connections.
#[derive(Debug)]
pub(super) struct ConnectionHandshake {
    context: HandshakeContext,
    state: HandshakeState,
    reliability: ReliabilityData,
    last_sent: Option<Instant>,
}

#[derive(Debug, Resource, Clone)]
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
}

impl ConnectionHandshake {
    pub fn new_incoming(
        context: HandshakeContext,
    ) -> Self {
        Self {
            context,
            state: HandshakeState::RelSynRecv,
            reliability: ReliabilityData::new(),
            last_sent: None,
        }
    }

    pub fn new_outgoing(
        context: HandshakeContext,
    ) -> Self {
        Self {
            context,
            state: HandshakeState::RelSynSent,
            reliability: ReliabilityData::new(),
            last_sent: None,
        }
    }

    pub fn poll(
        mut self,
        packets: &mut PacketQueue,
    ) -> PotentialStateTransition<Self, ()> {
        match self.state {
            HandshakeState::RelSynSent => {
                // Check if we've received a response yet
                if let Some(packet) = packets.pop_incoming() {
                    todo!()
                }

                // Check if we need to resend the packet
                let now = Instant::now();
                if timeout_check(self.last_sent, now, HANDSHAKE_RESEND_DURATION) {
                    // Generate packet and queue it for sending
                    let bytes = build_first_pkt(&self.context, &self.reliability);
                    packets.push_outgoing(OutgoingPacket {
                        payload: bytes,
                        messages: 0,
                    });

                    // Update time tracking info
                    self.last_sent = Some(now);
                }

                // Do nothing
                return PotentialStateTransition::Nothing(self);
            },
            HandshakeState::RelSynRecv => todo!(),
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

fn recv_first_pkt(
    context: &HandshakeContext,
    slice: &[u8],
) -> Result<u16, HandshakeResponseCode> {
    // Create reader to read through their data
    let mut reader = Reader::new(Input::from(slice));

    // Check the unique transport identifier
    let tp_id = u64::from_be_bytes(slice_to_array::<8>(&mut reader)?);
    if tp_id != context.transport_identifier {
        return Err(HandshakeResponseCode::RejectIncompatibleTransport)
    }

    // Check the transport major version
    let tp_maj_ver = u32::from_be_bytes(slice_to_array::<4>(&mut reader)?);
    if tp_maj_ver != context.transport_version_major {
        return Err(HandshakeResponseCode::RejectIncompatibleVersion)
    }

    // We don't check the minor version as of now.
    // At some point in the future, we can reject buggy versions because of it or something.

    // Check the application identifier
    let app_id = u64::from_be_bytes(slice_to_array::<8>(&mut reader)?);
    if app_id != context.application_identifier {
        return Err(HandshakeResponseCode::RejectIncompatibleApplication)
    }

    // Get their sequence identifier
    let seq_id = u16::from_be_bytes(slice_to_array::<2>(&mut reader)?);

    // Return relevant data
    return Ok(seq_id)
}

fn build_second_pkt_ok(
    context: &HandshakeContext,
    reliability: &ReliabilityData,
) -> Bytes {
    // Create buffer for storing bytes
    let mut buf = BytesMut::with_capacity(32);

    // Response code
    buf.put_u16(HandshakeResponseCode::Accept as u16);

    // Info about our app
    buf.put_u64(context.transport_identifier);
    buf.put_u32(context.transport_version_major);
    buf.put_u32(context.transport_version_minor);
    buf.put_u64(context.application_identifier);

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
    buf.put_u16(HandshakeResponseCode::Accept as u16);

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

struct HandshakeResponse  {
    pub code: HandshakeResponseCode,
    pub payload: Option<Bytes>,
}

#[repr(u16)]
enum HandshakeResponseCode {
    Accept = 0,
    RejectNoReason = 1,
    RejectHumanReason = 2,
    RejectBadPacket = 3,
    RejectIncompatibleTransport = 4,
    RejectIncompatibleVersion = 5,
    RejectIncompatibleApplication = 6,
}

impl TryFrom<u16> for HandshakeResponseCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Accept,
            _ => { return Err(()) }
        })
    }
}

impl From<EndOfInput> for HandshakeResponseCode {
    fn from(value: EndOfInput) -> Self {
        Self::RejectBadPacket
    }
}