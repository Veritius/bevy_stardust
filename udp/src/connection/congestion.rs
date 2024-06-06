use std::time::{Duration, Instant};

pub const DEFAULT_RESEND: Duration = Duration::from_secs(2);
pub const DEFAULT_MTU: usize = 1472;
pub const DEFAULT_BUDGET: usize = 16384;

const DEFAULT_REGENERATION: f32 = 4096.0;

/// Congestion controller.
pub(super) struct Congestion {
    resend_user_time: Duration,

    mtu_user_limit: usize,
    mtu_calc_limit: usize,

    bytes_user_limit: usize,
    bytes_cur_budget: usize,
    bytes_last_ticked: Instant,
    bytes_regeneration: f32,
}

impl Congestion {
    pub fn set_usr_resend(&mut self, time: Duration) {
        self.resend_user_time = time;
    }

    pub fn set_usr_mtu(&mut self, mtu: usize) {
        self.mtu_user_limit = mtu;
    }

    pub fn set_usr_budget(&mut self, budget: usize) {
        self.bytes_user_limit = budget;
    }

    pub fn get_resend(&self) -> Duration {
        self.resend_user_time
    }

    pub fn get_mtu(&self) -> usize {
        self.mtu_user_limit.min(self.mtu_calc_limit)
    }

    pub fn get_budget(&mut self, now: Instant) -> usize {
        let dur = self.bytes_last_ticked.duration_since(now);
        self.bytes_last_ticked = now;

        let delta = dur.as_secs_f32() * self.bytes_regeneration;
        self.bytes_cur_budget = (self.bytes_cur_budget + delta as usize)
            .min(self.bytes_user_limit);

        return self.bytes_cur_budget;
    }

    pub fn consume_budget(&mut self, amt: usize) {
        self.bytes_cur_budget = self.bytes_cur_budget.saturating_sub(amt);
    }
}

impl Default for Congestion {
    fn default() -> Self {
        Self {
            resend_user_time: DEFAULT_RESEND,

            mtu_user_limit: DEFAULT_MTU,
            mtu_calc_limit: DEFAULT_MTU,

            bytes_user_limit: DEFAULT_BUDGET,
            bytes_cur_budget: DEFAULT_BUDGET,
            bytes_last_ticked: Instant::now(),
            bytes_regeneration: DEFAULT_REGENERATION,
        }
    }
}