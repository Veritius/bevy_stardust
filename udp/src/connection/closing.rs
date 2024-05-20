use std::time::{Duration, Instant};
use bevy::prelude::*;
use bytes::Bytes;

#[derive(Component)]
pub(crate) struct Closing {
    started: Instant,
    timeout: Duration,

    reason: Option<Bytes>,
    inform: bool,

    finished: bool,
    informed: bool,
}

impl Closing {
    pub fn new(
        reason: Option<Bytes>,
        inform: bool,
    ) -> Self {
        Self {
            started: Instant::now(),
            timeout: Duration::from_secs(10),

            reason,
            inform,

            finished: false,
            informed: false,
        }
    }

    pub fn set_informed(&mut self) {
        self.informed = true;
    }

    pub fn needs_inform(&self) -> bool {
        !self.inform | self.informed
    }

    pub fn set_finished(&mut self) {
        self.finished = true;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }
}