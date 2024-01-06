use std::{time::Instant, net::SocketAddr};
use anyhow::Result;

pub(crate) struct IncomingConnectionAttempt {
    started: Instant,
    address: SocketAddr,
    last_sent: Instant,
}

impl IncomingConnectionAttempt {
    pub fn new(bytes: &[u8]) -> Result<Self> {
        todo!()
    }

    pub fn recv(&mut self, bytes: &[u8]) {
        todo!()
    }

    pub fn poll_send(&mut self, scratch: &mut [u8]) -> Option<usize> {
        todo!()
    }
}