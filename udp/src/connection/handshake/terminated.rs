use super::{codes::HandshakeResponseCode, ConnectionDirection};

#[derive(Debug)]
#[repr(transparent)]
pub(super) struct Terminated {
    reason: TerminationReason,
}

impl Terminated {
    pub fn reason(&self) -> TerminationReason {
        self.reason.clone()
    }
}

impl From<TerminationReason> for Terminated {
    #[inline]
    fn from(reason: TerminationReason) -> Self {
        Self { reason }
    }
}

#[derive(Debug, Clone)]
pub(super) struct TerminationReason {
    pub code: HandshakeResponseCode,
    pub origin: ConnectionDirection,
}