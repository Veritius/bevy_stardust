use bytes::{BufMut, Bytes};
use untrusted::*;
use crate::appdata::NetworkVersionData;
use super::codes::HandshakeResponseCode;

pub(super) trait HandshakePacket: Sized {
    fn from_reader(reader: &mut Reader) -> HandshakeParsingResponse<Self>;
    fn write_bytes(&self, buffer: &mut impl BufMut);
}

pub(super) enum HandshakeParsingResponse<T> {
    Continue(T),
    WeRejected(HandshakeResponseCode),
    TheyRejected(HandshakeResponseCode),
}

impl<T> From<EndOfInput> for HandshakeParsingResponse<T> {
    fn from(_: EndOfInput) -> Self {
        HandshakeParsingResponse::WeRejected(HandshakeResponseCode::MalformedPacket)
    }
}

/// On-wire format:
/// - \[u16] Sequence identifier
#[derive(Debug)]
pub(super) struct HandshakePacketHeader {
    pub sequence: u16,
}

/// Utility for sending a close signal to a connection.
#[derive(Debug)]
pub(super) struct ClosingPacket {
    pub header: HandshakePacketHeader,
    pub reason: HandshakeResponseCode,
    pub additional: Option<Bytes>,
}

/// Client sends its sequence id to establish reliability
/// 
/// On-wire format:
/// - \[xxx] Handshake packet header
/// - \[u16] Response code
/// - \[xxx] Transport version data
/// - \[xxx] Application version data
#[derive(Debug)]
pub(super) struct ClientHelloPacket {
    pub transport: NetworkVersionData,
    pub application: NetworkVersionData,
}

/// Server acknowledges the client's request (ClientHelloPacket)
/// Server sends its sequence id to establish reliability
/// 
/// On-wire format:
/// - \[xxx] Handshake packet header
/// - \[u16] Response code
/// - \[xxx] Transport version data
/// - \[xxx] Application version data
/// - \[u16] Reliability ack
/// - \[u16] Reliability bitfield
#[derive(Debug)]
pub(super) struct ServerHelloPacket {
    pub transport: NetworkVersionData,
    pub application: NetworkVersionData,
    pub reliability_ack: u16,
    pub reliability_bits: u16,
}

/// Client acknowledges the server's response (ServerHelloPacket)
/// 
/// On-wire format:
/// - \[xxx] Handshake packet header
/// - \[u16] Response code
/// - \[u16] Reliability ack
/// - \[u16] Reliability bitfield
#[derive(Debug)]
pub(super) struct ClientFinalisePacket {
    pub reliability_ack: u16,
    pub reliability_bits: u16,
}