use std::{net::SocketAddr, time::{Duration, Instant}};
use bytes::Bytes;
use super::{AttemptOutcome, HandshakeFailure};

pub(crate) struct PendingOutgoing {
    pub remote: SocketAddr,
    started: Instant,
    timeout: Duration,
    state: OutgoingState,
}

impl PendingOutgoing {
    pub fn new() {

    }

    pub fn recv(&mut self, bytes: &[u8]) {
        todo!()
    }

    pub fn send(&mut self, scratch: &mut [u8]) -> usize {
        todo!()
    }

    pub fn finished(&self) -> bool {
        match self.state {
            OutgoingState::Complete => true,
            OutgoingState::Failed(_) => true,
            _ => false,
        }
    }

    pub fn finish(self) -> AttemptOutcome {
        match self.state {
            OutgoingState::Complete => todo!(),
            OutgoingState::Failed(failure) => AttemptOutcome::Failure(failure),
            _ => AttemptOutcome::Unfinished,
        }
    }
}

enum OutgoingState {
    WaitingResponse {
        last_sent: Instant,
    },
    PendingSend {
        message: Bytes,
    },
    Complete,
    Failed(HandshakeFailure),
}