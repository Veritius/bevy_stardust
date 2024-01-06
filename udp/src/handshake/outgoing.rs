use std::{time::Instant, net::SocketAddr};

pub(crate) struct OutgoingConnectionAttempt {
    started: Instant,
    address: SocketAddr,
}

impl OutgoingConnectionAttempt {
    pub fn new(address: SocketAddr) -> Self {
        todo!()
    }

    pub fn recv(&mut self, bytes: &[u8]) {
        todo!()
    }

    pub fn poll_send(&mut self, scratch: &mut [u8]) -> Option<usize> {
        todo!()
    }
}