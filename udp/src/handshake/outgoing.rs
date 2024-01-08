use bytes::Bytes;

pub(super) struct OutgoingAttemptData {
    state: OutgoingState,
}

impl Default for OutgoingAttemptData {
    fn default() -> Self {
        Self {
            state: OutgoingState::Initial,
        }
    }
}

impl OutgoingAttemptData {
    pub fn poll_send(&mut self, scratch: &mut [u8]) -> Option<usize> {
        todo!()
    }

    pub fn recv(&mut self, data: Bytes) {
        todo!()
    }
}

enum OutgoingState {
    Initial,
}