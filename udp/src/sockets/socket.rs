use std::net::UdpSocket;

pub(crate) struct Socket {
    socket: UdpSocket,
}