use std::net::SocketAddr;
use bytes::Bytes;

/// A datagram that must be transmitted.
pub struct Transmit {
    pub remote: SocketAddr,
    pub data: Bytes,
}