use std::sync::Arc;
use bevy::prelude::*;
use smallvec::{smallvec, SmallVec};

/// Filtering method.
pub enum MembershipFilter {
    /// Replicated to peers that are members in at least one of the contained groups.
    Inclusive(SmallVec<[Entity; 4]>),

    /// Replicated to peers that are not members of any of the contained groups.
    Exclusive(SmallVec<[Entity; 4]>),

    /// Use a custom function for filtering.
    /// `true` means that the target is replicated in the room with the passed entity ID.
    Custom(Arc<dyn Fn(Entity) -> bool + Send + Sync>),
}

impl MembershipFilter {
    fn rm_many(vec: &mut SmallVec<[Entity; 4]>, item: Entity) {
        let el = vec.iter()
            .enumerate()
            .filter(|(_, e)| **e == item)
            .map(|v| v.0)
            .collect::<SmallVec<[usize; 8]>>();

        for idx in el.iter() {
            vec.remove(*idx);
        }
    }

    /// Try to add `room` to the set such that in [`filter`](Self::filter) will return `true` when passed `room`.
    /// 
    /// Returns `false` if this was impossible.
    /// This currently only occurs if the variant is [`Custom`][MembershipFilter::Custom].
    pub fn include(&mut self, room: Entity) -> bool {
        match self {
            MembershipFilter::Inclusive(vec) => {
                if vec.contains(&room) { return true; }
                vec.push(room);
                return true;
            },
            MembershipFilter::Exclusive(vec) => {
                Self::rm_many(vec, room);
                return true;
            },
            MembershipFilter::Custom(_) => {
                // Not possible
                return false;
            },
        }
    }

    /// Try to add `room` to the set such that in [`filter`](Self::filter) will return `false` when passed `room`.
    /// 
    /// Returns `false` if this was impossible.
    /// This currently only occurs if the variant is [`Custom`][MembershipFilter::Custom].
    pub fn exclude(&mut self, room: Entity) -> bool {
        match self {
            MembershipFilter::Inclusive(vec) => {
                Self::rm_many(vec, room);
                return true;
            },
            MembershipFilter::Exclusive(vec) => {
                if vec.contains(&room) { return true; }
                vec.push(room);
                return true;
            },
            MembershipFilter::Custom(_) => {
                // Not possible
                return false;
            },
        }
    }

    /// Returns `true` if group matches the filter.
    pub fn filter(&self, group: Entity) -> bool {
        self.filter_inlined(group)
    }

    /// Returns `true` if `group` matches the filter.
    /// This function is inlined - use [`filter`](Self::filter) if you don't want this.
    #[inline]
    pub(crate) fn filter_inlined(&self, group: Entity) -> bool {
        // TODO: Maybe ensure vecs are sorted so binary search can be used
        match self {
            MembershipFilter::Inclusive(set) => set.contains(&group),
            MembershipFilter::Exclusive(set) => !set.contains(&group),
            MembershipFilter::Custom(func) => func(group),
        }
    }
}

impl Default for MembershipFilter {
    fn default() -> Self {
        Self::Inclusive(smallvec![])
    }
}