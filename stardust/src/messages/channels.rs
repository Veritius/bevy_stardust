use super::MessageConsistency;

#[cfg(feature="reflect")]
use bevy_reflect::Reflect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature="reflect", derive(Reflect))]
pub struct MessageChannelId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature="reflect", derive(Reflect))]
pub struct MessageChannelPersistentId(u64);

pub trait MessageChannel: 'static {
    const PRIORITY: i32;

    const CONSISTENCY: MessageConsistency;

    #[inline]
    fn is_reliable() -> bool {
        Self::CONSISTENCY.is_reliable()
    }

    #[inline]
    fn is_ordered() -> bool {
        Self::CONSISTENCY.is_ordered()
    }
}