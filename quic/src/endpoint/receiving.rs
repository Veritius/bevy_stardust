use std::{io::{Error as IoError, ErrorKind}, net::{SocketAddr, UdpSocket}};

/// A handle to a UDP socket.
pub struct UdpSocketRecv<'a> {
    socket: &'a UdpSocket,
    scratch: &'a mut [u8],
}

impl<'a> UdpSocketRecv<'a> {
    /// Try to receive packets, with three possible cases:
    /// - `Ok(Some())` - A UDP packet was received over the socket
    /// - `Ok(None)` - No more packets are available this tick
    /// - `Err()` - An I/O error occurred
    pub fn recv(&mut self) -> Result<Option<ReceivedDatagram>, IoError> {
        match self.socket.recv_from(&mut self.scratch) {
            Ok((length, address)) => {
                let payload = &self.scratch[..length];
                return Ok(Some(ReceivedDatagram { address, payload }));
            },

            Err(err) if err.kind() == ErrorKind::WouldBlock => return Ok(None),

            Err(err) => return Err(err),
        }
    }
}

/// A datagram that has been received.
pub struct ReceivedDatagram<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}