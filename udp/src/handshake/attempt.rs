use std::{net::SocketAddr, time::{Instant, Duration}};
use bytes::Bytes;
use crate::{reliability::ReliableRiver, connection::UdpConnection};
use super::{error::HandshakeError, outgoing::OutgoingAttemptData, incoming::IncomingAttemptData};

pub(crate) struct ConnectionAttempt {
    shared: AttemptShared,
    direction: AttemptDirection,
}

impl ConnectionAttempt {
    pub fn start_outgoing(
        address: SocketAddr,
        timeout: Duration,
    ) -> Self {
        Self {
            shared: AttemptShared {
                address,
                started: Instant::now(),
                timeout,
                last_sent: None,
                last_recv: None,
                river: ReliableRiver::new(),
            },
            direction: AttemptDirection::Outgoing(
                OutgoingAttemptData::default()
            )
        }
    }

    pub fn start_incoming(
        address: SocketAddr,
        timeout: Duration,
        payload: &[u8],
    ) -> Result<Self, HandshakeError> {
        todo!()
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

pub(super) struct AttemptShared {
    address: SocketAddr,
    started: Instant,
    timeout: Duration,
    last_sent: Option<Instant>,
    last_recv: Option<Instant>,
    river: ReliableRiver,
}

enum AttemptDirection {
    Outgoing(OutgoingAttemptData),
    Incoming(IncomingAttemptData),
}