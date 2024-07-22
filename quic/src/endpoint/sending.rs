use std::io::Error as IoError;
use std::net::{SocketAddr, UdpSocket};

/// A handle to a UDP socket.
pub struct UdpSocketSend<'a> {
    socket: &'a UdpSocket,
}

impl<'a> UdpSocketSend<'a> {
    /// Try to send a UDP packet over the socket.
    pub fn send(&mut self, transmit: TransmitDatagram) -> Result<(), IoError> {
        match self.socket.send_to(transmit.payload, transmit.address) {
            Ok(_) => return Ok(()),
            Err(err) => return Err(err),
        }
    }
}

/// A datagram that must be transmitted.
pub struct TransmitDatagram<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}