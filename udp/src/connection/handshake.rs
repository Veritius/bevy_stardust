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
use crate::{appdata::{ApplicationContext, TRANSPORT_IDENTIFIER, TRANSPORT_VERSION_MAJOR, TRANSPORT_VERSION_MINOR}, packet::{OutgoingPacket, PacketQueue}, utils::slice_to_array};
use super::{reliability::{ReliabilityData, ReliablePacketHeader}, timing::timeout_check};

const HANDSHAKE_RESEND_DURATION: Duration = Duration::from_secs(5);

/// Handshake state machine for connections.
#[derive(Debug)]
pub(super) struct ConnectionHandshake {
    context: ApplicationContext,
    side: HandshakeSide,
    reliability: ReliabilityData,
    last_sent: Option<Instant>,
}

#[derive(Debug)]
enum HandshakeSide {
    Client,
    Server,
}

impl ConnectionHandshake {
    pub fn new_incoming(
        context: ApplicationContext,
        reader: &mut Reader,
    ) -> Result<Self, HandshakeFailure> {
        // Parse the first packet
        match recv_first_pkt(&context, reader) {
            Ok(seq) => {
                // Set up reliability
                let mut reliability = ReliabilityData::new();
                reliability.remote_sequence = seq;
                reliability.local_sequence = fastrand::u16(..);

                // Return new handshake
                return Ok(Self {
                    context,
                    side: HandshakeSide::Server,
                    reliability,
                    last_sent: None,
                });
            },

            Err(err) => { Err(err) },
        }
    }

    pub fn new_outgoing(
        context: ApplicationContext,
    ) -> Self {
        Self {
            context,
            side: HandshakeSide::Client,
            reliability: ReliabilityData::new(),
            last_sent: None,
        }
    }

    pub fn poll(
        mut self,
        packets: &mut PacketQueue,
    ) -> HandshakePollOutcome {
        match self.side {
            HandshakeSide::Client => {
                // Check if we've received a response yet
                if let Some(packet) = packets.pop_incoming() {
                    let mut reader = Reader::new(Input::from(&packet.payload));

                    // Parse packet to see if it's valid
                    match recv_second_pkt(&self.context, &mut reader) {
                        // Valid packet
                        PacketRecvOutcome::Valid(header) => {
                            // Update reliability state
                            self.reliability.remote_sequence = header.sequence;
                            self.reliability.local_sequence = self.reliability.local_sequence.wrapping_add(1);

                            // Send response packet (third)
                            let payload = build_third_pkt_ok(&self.reliability);
                            packets.push_outgoing(OutgoingPacket {
                                payload,
                                messages: 0,
                            });

                            // Transition into finished state
                            return HandshakePollOutcome::Success(self.reliability);
                        },

                        // Invalid packet
                        PacketRecvOutcome::Failure(reason) => {
                            // Return the failure reason
                            return HandshakePollOutcome::Failure(reason);
                        },
                    }
                }

                // Check if we need to send a response packet
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
                return HandshakePollOutcome::Waiting(self);
            },

            HandshakeSide::Server => {
                // Check if we've received a response yet
                if let Some(packet) = packets.pop_incoming() {
                    let mut reader = Reader::new(Input::from(&packet.payload));

                    // Parse packet to see if it's valid
                    match recv_third_pkt(&mut reader) {
                        PacketRecvOutcome::Valid(header) => {
                            // Update reliability state
                            self.reliability.remote_sequence = header.sequence;

                            // Transition into finished state
                            return HandshakePollOutcome::Success(self.reliability);
                        },

                        PacketRecvOutcome::Failure(reason) => {
                            // Return the failure reason
                            return HandshakePollOutcome::Failure(reason);
                        },
                    }
                }

                // Check if we need to send a response packet
                let now = Instant::now();
                if self.last_sent.is_none() {
                    // Generate packet and queue it for sending
                    let bytes = build_second_pkt_ok(&self.context, &self.reliability);
                    packets.push_outgoing(OutgoingPacket {
                        payload: bytes,
                        messages: 0,
                    });

                    // Update time tracking info
                    self.last_sent = Some(now);
                }

                // Do nothing
                return HandshakePollOutcome::Waiting(self);
            },
        }
    }
}

fn build_first_pkt(
    context: &ApplicationContext,
    reliability: &ReliabilityData,
) -> Bytes {
    // Create buffer for storing bytes
    let mut buf = BytesMut::with_capacity(26);

    // Info about our app
    buf.put_u64(TRANSPORT_IDENTIFIER);
    buf.put_u32(TRANSPORT_VERSION_MAJOR);
    buf.put_u32(TRANSPORT_VERSION_MINOR);
    buf.put_u64(context.application_identifier);

    // Info about reliability
    buf.put_u16(reliability.local_sequence);

    // Make immutable and return
    return buf.freeze();
}

fn recv_first_pkt(
    context: &ApplicationContext,
    reader: &mut Reader,
) -> Result<u16, HandshakeFailure> {
    // Get the transport identifier
    let tp_id = u64::from_be_bytes(match slice_to_array::<8>(reader) {
        Ok(ident) => ident,
        Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
    });

    // Check the transport identifier
    if tp_id != TRANSPORT_IDENTIFIER {
        return HandshakeFailure::us(HandshakeResponseCode::RejectIncompatibleTransport).into();
    }

    // We don't check the minor version as of now.
    // At some point in the future, we can reject buggy versions because of it or something.

    // Get the major transport version
    let tp_maj_ver = u32::from_be_bytes(match slice_to_array::<4>(reader) {
        Ok(ident) => ident,
        Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
    });

    // Check the major transport version
    if tp_maj_ver != TRANSPORT_VERSION_MAJOR {
        return HandshakeFailure::us(HandshakeResponseCode::RejectIncompatibleVersion).into();
    }

    // Get the application identifier
    let app_id = u64::from_be_bytes(match slice_to_array::<8>(reader) {
        Ok(ident) => ident,
        Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
    });

    // Check the application identifier
    if app_id != context.application_identifier {
        return HandshakeFailure::us(HandshakeResponseCode::RejectIncompatibleApplication).into();
    }

    // Get their sequence identifier
    let seq_id = u16::from_be_bytes(match slice_to_array::<2>(reader) {
        Ok(val) => val,
        Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
    });

    // Return relevant data
    return Ok(seq_id)
}

fn build_second_pkt_ok(
    context: &ApplicationContext,
    reliability: &ReliabilityData,
) -> Bytes {
    // Create buffer for storing bytes
    let mut buf = BytesMut::with_capacity(32);

    // Response code
    buf.put_u16(HandshakeResponseCode::Accept as u16);

    // Info about our app
    buf.put_u64(TRANSPORT_IDENTIFIER);
    buf.put_u32(TRANSPORT_VERSION_MAJOR);
    buf.put_u32(TRANSPORT_VERSION_MINOR);
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

fn recv_second_pkt(
    context: &ApplicationContext,
    reader: &mut Reader,
) -> PacketRecvOutcome<ReliablePacketHeader> {
    // TODO: When try_trait_v2 stabilises, use FromResidual to make this code more concise

    // Get the response code from the packet
    // If this fails, we just say their packet was invalid
    let response_code = match resp_code_qfx(reader) {
        Some(code) => code,
        None => { return HandshakeFailure::them(HandshakeResponseCode::RejectBadPacket).into(); }
    };

    // Check the response code
    if response_code != HandshakeResponseCode::Accept {
        return HandshakeFailure::them(response_code).into();
    }

    // Get the transport identifier
    let tp_id = u64::from_be_bytes(match slice_to_array::<8>(reader) {
        Ok(ident) => ident,
        Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
    });

    // Check the transport identifier
    if tp_id != TRANSPORT_IDENTIFIER {
        return HandshakeFailure::us(HandshakeResponseCode::RejectIncompatibleTransport).into()
    }

    // Get the major transport version
    let tp_maj_ver = u32::from_be_bytes(match slice_to_array::<4>(reader) {
        Ok(ident) => ident,
        Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
    });

    // Check the major transport version
    if tp_maj_ver != TRANSPORT_VERSION_MAJOR {
        return HandshakeFailure::us(HandshakeResponseCode::RejectIncompatibleVersion).into();
    }

    // We don't check the minor version here either
    // See recv_first_pkt for an explanation why

    // Get the application identifier
    let app_id = u64::from_be_bytes(match slice_to_array::<8>(reader) {
        Ok(ident) => ident,
        Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
    });

    // Check the application identifier
    if app_id != context.application_identifier {
        return HandshakeFailure::us(HandshakeResponseCode::RejectIncompatibleApplication).into();
    }

    // Get reliability values from the packet
    let mut return_value = [0u16; 3];
    for i in 0..3 {
        let val = u16::from_be_bytes(match slice_to_array::<2>(reader) {
            Ok(ident) => ident,
            Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
        });

        return_value[i] = val;
    }

    // We're done, return our values
    return PacketRecvOutcome::Valid(ReliablePacketHeader {
        sequence: return_value[0],
        ack: return_value[1],
        ack_bitfield: extend_bitfield(return_value[2]),
    })
}

fn build_third_pkt_ok(
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

fn recv_third_pkt(
    reader: &mut Reader,
) -> PacketRecvOutcome<ReliablePacketHeader> {
    // TODO: When try_trait_v2 stabilises, use FromResidual to make this code more concise

    // Get the response code from the packet
    // If this fails, we just say their packet was invalid
    let response_code = match resp_code_qfx(reader) {
        Some(code) => code,
        None => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); }
    };

    // Get reliability values from the packet
    let mut return_value = [0u16; 3];
    for i in 0..3 {
        let val = u16::from_be_bytes(match slice_to_array::<2>(reader) {
            Ok(ident) => ident,
            Err(_) => { return HandshakeFailure::us(HandshakeResponseCode::RejectBadPacket).into(); },
        });

        return_value[i] = val;
    }

    // We're done, return our values
    return PacketRecvOutcome::Valid(ReliablePacketHeader {
        sequence: return_value[0],
        ack: return_value[1],
        ack_bitfield: extend_bitfield(return_value[2]),
    })
}

struct HandshakeResponse  {
    pub code: HandshakeResponseCode,
    pub payload: Option<Bytes>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)] // frees us from having to implement From<HandshakeResponseCode> for u16
pub(super) enum HandshakeResponseCode {
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

fn resp_code_qfx(reader: &mut Reader) -> Option<HandshakeResponseCode> {
    // Get the response code as an integer
    let response_code_int = u16::from_be_bytes(match slice_to_array::<2>(reader) {
        Ok(int) => int,
        Err(_) => { return None; },
    });

    // Turn integer into a response code enum variant
    let response_code = match HandshakeResponseCode::try_from(response_code_int) {
        Ok(code) => code,
        Err(_) => { return None; },
    };

    // Success!
    return Some(response_code)
}

fn extend_bitfield(val: u16) -> u128 {
    let mut arr = 0u128.to_ne_bytes();
    let spl = val.to_ne_bytes();
    arr[0] = spl[0];
    arr[1] = spl[1];

    return u128::from_ne_bytes(arr);
}

enum PacketRecvOutcome<T> {
    Valid(T),
    Failure(HandshakeFailure),
}

#[derive(Debug)]
pub(super) struct HandshakeFailure {
    pub side: HandshakeFailureSide,
    pub code: HandshakeResponseCode,
}

impl HandshakeFailure {
    fn them(code: HandshakeResponseCode) -> Self {
        Self {
            side: HandshakeFailureSide::Them,
            code,
        }
    }

    fn us(code: HandshakeResponseCode) -> Self {
        Self {
            side: HandshakeFailureSide::Us,
            code,
        }
    }
}

impl<T> From<HandshakeFailure> for PacketRecvOutcome<T> {
    fn from(value: HandshakeFailure) -> Self {
        Self::Failure(value)
    }
}

impl<T> From<HandshakeFailure> for Result<T, HandshakeFailure> {
    fn from(value: HandshakeFailure) -> Self {
        Self::Err(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HandshakeFailureSide {
    Us,
    Them,
}

pub(super) enum HandshakePollOutcome {
    Waiting(ConnectionHandshake),
    Success(ReliabilityData),
    Failure(HandshakeFailure),
}