use std::{net::SocketAddr, time::{Duration, Instant}};
use bytes::Bytes;
use super::{AttemptOutcome, HandshakeFailure};

pub(crate) struct PendingIncoming {
    pub remote: SocketAddr,
    started: Instant,
    timeout: Duration,
    state: IncomingState,
}

impl PendingIncoming {
    pub fn new(bytes: &[u8]) -> Result<Self, HandshakeFailure> {
        todo!()
    }

    pub fn recv(&mut self, bytes: &[u8]) {
        todo!()
    }

    pub fn send(&mut self, scratch: &mut [u8]) -> usize {
        todo!()
    }

    pub fn finished(&self) -> bool {
        match self.state {
            IncomingState::Complete => true,
            IncomingState::Failed(_) => true,
            _ => false,
        }
    }

    pub fn finish(self) -> AttemptOutcome {
        match self.state {
            IncomingState::Complete => todo!(),
            IncomingState::Failed(failure) => AttemptOutcome::Failure(failure),
            _ => AttemptOutcome::Unfinished,
        }
    }
}

enum IncomingState {
    PendingSend {
        message: Bytes,
    },
    WaitingResponse {
        last_sent: Instant,
    },
    Complete,
    Failed(HandshakeFailure),
}