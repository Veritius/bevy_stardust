use std::collections::BTreeMap;

use super::MessageConsistency;

#[cfg(feature="reflect")]
use bevy_reflect::Reflect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature="reflect", derive(Reflect))]
pub struct MessageChannelId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature="reflect", derive(Reflect))]
pub struct MessageChannelPersistentId(pub u64);

pub trait MessageChannel: 'static {
    const IDENTIFIER: MessageChannelPersistentId;

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

pub struct MessageChannelIds {
    index: usize,

    transient: BTreeMap<MessageChannelPersistentId, MessageChannelId>,
    persistent: BTreeMap<MessageChannelId, MessageChannelPersistentId>,
}