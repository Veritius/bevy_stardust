use std::time::Instant;

#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub(crate) struct ConnectionTimings {
    pub last_sent: Option<Instant>,
    pub last_recv: Option<Instant>,
}

impl ConnectionTimings {
    pub fn new(
        last_sent: Option<Instant>,
        last_recv: Option<Instant>,
    ) -> Self {
        Self {
            last_sent,
            last_recv,
        }
    }
}