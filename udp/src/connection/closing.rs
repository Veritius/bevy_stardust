use bytes::Bytes;

pub(super) struct CloseRequest {
    closing: bool,
    finished: bool,
    reason: Option<Bytes>,
}

impl CloseRequest {
    pub fn new() -> Self {
        Self {
            closing: false,
            finished: true,
            reason: None,
        }
    }

    pub fn begin_local_close(&mut self, reason: Option<Bytes>) {
        if self.closing { return }
        self.closing = true;
        self.reason = reason;
    }

    pub fn begin_remote_close(&mut self, reason: Option<Bytes>) {
        if self.closing { return }
        self.closing = true;
        self.reason = reason;
    }

    pub fn finish_close(&mut self) {
        self.finished = true;
    }

    pub fn is_closing(&self) -> bool {
        self.closing
    }

    pub fn is_closed(&self) -> bool {
        self.finished
    }

    pub fn reason(&self) -> Option<Bytes> {
        self.reason.clone()
    }
}