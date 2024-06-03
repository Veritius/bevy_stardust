use bytes::Bytes;
use tracing::error;

#[derive(Debug)]
pub(super) struct ClosingManager {
    closing: bool,
    finished: bool,
    informed: bool,
    origin: Origin,
    reason: Option<Bytes>,
}

impl Default for ClosingManager {
    fn default() -> Self {
        Self {
            closing: false,
            finished: false,
            informed: false,
            origin: Origin::Local, // this is overwritten anyway
            reason: None,
        }
    }
}

impl ClosingManager {
    pub fn start_close(
        &mut self,
        origin: Origin,
        reason: Option<Bytes>,
    ) {
        if self.closing { return }
        self.closing = true;
        self.origin = origin;
        self.reason = reason;
    }

    pub fn finish_close(&mut self) {
        assert!(self.closing); // TODO: Don't panic
        self.finished = true;
    }

    pub fn inform(&mut self, call: impl FnOnce()) {
        if !self.closing || self.informed { return }
        self.informed = true;
        call()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
    Local,
    Remote,
}