use bytes::BufMut;
use untrusted::*;
use crate::appdata::NetworkVersionData;

pub(super) trait HandshakePacket {
    fn from_reader(reader: &mut Reader);
    fn write_bytes(buffer: &mut impl BufMut);
}

/// On-wire format:
/// - \[u16] Sequence identifier
#[derive(Debug)]
pub(super) struct HandshakePacketHeader {
    pub sequence: u16,
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
    pub header: HandshakePacketHeader,
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
    pub header: HandshakePacketHeader,
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
    pub header: HandshakePacketHeader,
    pub reliability_ack: u16,
    pub reliability_bits: u16,
}