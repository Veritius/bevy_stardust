use std::{io::ErrorKind, net::{SocketAddr, UdpSocket}};
use anyhow::Result;

/// A handle to a UDP socket.
pub struct UdpSocketRecv<'a> {
    socket: &'a UdpSocket,
    scratch: &'a mut [u8],
}

impl<'a> UdpSocketRecv<'a> {
    pub fn recv(&mut self) -> Option<Result<ReceivedDatagram>> {
        match self.socket.recv_from(&mut self.scratch) {
            Ok((length, address)) => {
                let payload = &self.scratch[..length];
                return Some(Ok(ReceivedDatagram { address, payload }));
            },

            Err(err) if err.kind() == ErrorKind::WouldBlock => return None,

            Err(err) => return Some(Err(<std::io::Error as Into<anyhow::Error>>::into(err)
                .context("while receiving udp packets"))),
        }
    }
}

/// A datagram that has been received.
pub struct ReceivedDatagram<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}