use std::time::{Duration, Instant};

pub const DEFAULT_RESEND: Duration = Duration::from_secs(2);
pub const DEFAULT_MTU: usize = 1472;
pub const DEFAULT_KBPS: usize = 512; // 512kbps

/// Congestion controller.
pub(super) struct Congestion {
    resend_user_time: Duration,

    mtu_user_limit: usize,
    // mtu_calc_limit: usize,

    bytes_user_kbps: usize,
    bytes_cur_budget: usize,
    bytes_last_ticked: Instant,
}

impl Congestion {
    pub fn set_resend(&mut self, time: Duration) {
        self.resend_user_time = time;
    }

    pub fn set_mtu(&mut self, mtu: usize) {
        self.mtu_user_limit = mtu;
    }

    pub fn set_kbps(&mut self, kbps: usize) {
        self.bytes_user_kbps = kbps;
    }

    pub fn get_resend(&self) -> Duration {
        self.resend_user_time
    }

    pub fn get_mtu(&self) -> usize {
        self.mtu_user_limit//.min(self.mtu_calc_limit)
    }

    pub fn get_budget(&mut self, now: Instant) -> usize {
        let dur = now.duration_since(self.bytes_last_ticked);
        self.bytes_last_ticked = now;

        let delta = (dur.as_millis() as usize / 10) * self.bytes_user_kbps;
        self.bytes_cur_budget = (self.bytes_cur_budget + delta)
            .min(self.bytes_user_kbps);

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
            // mtu_calc_limit: DEFAULT_MTU,

            bytes_user_kbps: DEFAULT_KBPS,
            bytes_cur_budget: DEFAULT_KBPS,
            bytes_last_ticked: Instant::now(),
        }
    }
}