use bytes::Bytes;

pub(super) struct IncomingAttemptData {
    state: IncomingState,
}

impl Default for IncomingAttemptData {
    fn default() -> Self {
        Self {
            state: IncomingState::Initial,
        }
    }
}

impl IncomingAttemptData {
    pub fn poll_send(&mut self, scratch: &mut [u8]) -> Option<usize> {
        todo!()
    }

    pub fn recv(&mut self, data: Bytes) {
        todo!()
    }
}

enum IncomingState {
    Initial,
}