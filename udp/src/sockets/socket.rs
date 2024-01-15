use std::net::UdpSocket;

pub(crate) struct Socket {
    socket: UdpSocket,
}

impl Socket {
    pub(super) fn new(socket: UdpSocket) -> Self {
        Self {
            socket
        }
    }
}