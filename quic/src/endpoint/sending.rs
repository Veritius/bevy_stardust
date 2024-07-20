use std::net::{SocketAddr, UdpSocket};
use anyhow::Result;

/// A handle to a UDP socket.
pub struct UdpSocketSend<'a> {
    socket: &'a UdpSocket,
}

impl<'a> UdpSocketSend<'a> {
    pub fn send(&mut self, transmit: TransmitDatagram) -> Result<()> {
        match self.socket.send_to(transmit.payload, transmit.address) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}

/// A datagram that must be transmitted.
pub struct TransmitDatagram<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}