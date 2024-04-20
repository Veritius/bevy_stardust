use std::time::Instant;
use bevy::prelude::*;

#[derive(Reflect)]
#[reflect(from_reflect = false)]
pub(crate) struct ConnectionTimings {
    pub started: Instant,
    pub last_sent: Option<Instant>,
    pub last_recv: Option<Instant>,
}

impl ConnectionTimings {
    pub fn new(
        started: Option<Instant>,
        last_sent: Option<Instant>,
        last_recv: Option<Instant>,
    ) -> Self {
        Self {
            started: started.unwrap_or(Instant::now()),
            last_sent,
            last_recv,
        }
    }

    pub fn set_last_sent_now(&mut self) {
        self.last_sent = Some(Instant::now());
    }

    pub fn set_last_recv_now(&mut self) {
        self.last_recv = Some(Instant::now());
    }
}