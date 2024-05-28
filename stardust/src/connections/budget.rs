use std::sync::atomic::{AtomicU32, Ordering};
use bevy::prelude::*;

// TODO: All atomic operations here use the SeqCst ordering.
// Find out if any other operations may be better.

/// A send budget for a [`NetworkPeer`][crate::prelude::NetworkPeer] entity, in bytes.
/// This allows transport plugins to inform game systems of
/// the amount of data that can be sent this tick.
/// 
/// A `NetworkPeer` entity without this component can be assumed to have no budget,
/// and that an unlimited amount of information can be sent to them.
/// 
/// # Usage
/// Using part of the budget is done with [`try_use`](Self::try_use).
/// This doesn't require a mutable reference, as it uses atomics.
/// Change detection cannot detect this and is irrelevant for this component.
/// See the function documentation for more information.
/// 
/// Resetting the budget is done with [`set`](Self::set).
/// This should only be used by transport layers, and requires a mutable reference.
/// Using this function should only be done after the `NetworkWrite::Send` system set.
#[derive(Debug, Component)]
pub struct PeerSendBudget {
    num: AtomicU32,
}

impl PeerSendBudget {
    /// Creates a new `PeerSendBudget` component.
    pub const fn new(value: u32) -> Self {
        Self {
            num: AtomicU32::new(value)
        }
    }

    /// Sets the budget amount, used by transport layers.
    /// This should only be used in the following situations:
    /// - In the `First` schedule, depending on your application.
    /// - After the `NetworkWrite::Send` system set in `PostUpdate`.
    #[inline]
    pub fn set(&mut self, amount: u32) {
        *(self.num.get_mut()) = amount;
    }

    /// Returns how much is left in the budget at this point in time.
    /// This is exposed for debugging. You might want to use [`try_use`](Self::try_use) instead.
    #[inline]
    pub fn get(&mut self) -> u32 {
        *(self.num.get_mut())
    }

    /// Try to use `amount` bytes of the budget, returning `true` if there is enough remaining.
    /// If `true`, `amount` is subtracted from the budget. If `false`, nothing happens.
    /// 
    /// ```
    /// # fn main() {
    /// # use bevy_stardust::prelude::PeerSendBudget;
    /// let mut budget = PeerSendBudget::new(256);
    /// assert_eq!(budget.try_use(64), true);
    /// assert_eq!(budget.try_use(192), true);
    /// assert_eq!(budget.try_use(32), false);
    /// # }
    /// ```
    pub fn try_use(&self, amount: u32) -> bool {
        use Ordering::SeqCst;
        self.num.fetch_update(SeqCst, SeqCst, |old| {
            old.checked_sub(amount)
        }).is_ok()
    }
}