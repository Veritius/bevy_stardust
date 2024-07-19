use std::net::SocketAddr;
use anyhow::Result;
use bytes::Bytes;

/// An endpoint associated with a [`Backend`](crate::backend::Backend) implementation.
pub trait EndpointBackend
where
    Self: Send + Sync,
{
    /// Called when a new UDP packet is received.
    /// 
    /// `from` is the IP address and port the packet was sent from.
    /// `packet` is a slice containing the full received data.
    fn recv_udp_packet(&mut self, from: SocketAddr, packet: &[u8]) -> Result<()>;

    /// Called to see if the backend wants to transmit any new packets.
    fn send_udp_packet(&mut self) -> Option<Result<Transmit>>;
}

/// A datagram that must be transmitted.
pub struct Transmit {
    pub remote: SocketAddr,
    pub data: Bytes,
}