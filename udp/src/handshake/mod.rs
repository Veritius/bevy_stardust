use std::{net::SocketAddr, time::{Instant, Duration}};
use bytes::Bytes;
use crate::{reliability::ReliableRiver, connection::UdpConnection};

pub(crate) struct ConnectionAttempt {
    address: SocketAddr,
    started: Instant,
    timeout: Duration,
    last_sent: Option<Instant>,
    last_recv: Option<Instant>,
    river: ReliableRiver,
}

impl ConnectionAttempt {
    pub fn start_outgoing(
        address: SocketAddr,
        timeout: Duration,
    ) -> Self {
        Self {
            address,
            started: Instant::now(),
            timeout,
            last_sent: None,
            last_recv: None,
            river: ReliableRiver::new(),
        }
    }

    pub fn poll_send(&mut self, scratch: &mut [u8]) -> Option<usize> {
        todo!()
    }

    pub fn recv(&mut self, data: Bytes) {
        todo!()
    }

    pub fn ready_for_complete(&self) -> bool {
        todo!()
    }

    pub fn try_complete(self) -> Result<UdpConnection, Self> {
        todo!()
    }
}