use std::{collections::BTreeMap, fmt::Debug};

use crate::prelude::ChannelId;

/// Ordering data for a single peer.
#[derive(Debug)]
pub(super) struct Ordering {
    channels: BTreeMap<ChannelId, ChannelOrdering>
}

impl Default for Ordering {
    fn default() -> Self {
        Self { channels: Default::default() }
    }
}

/// Ordering data for a single channel.
pub(super) struct ChannelOrdering {
    
}

impl Default for ChannelOrdering {
    fn default() -> Self {
        Self {
            
        }
    }
}

impl Debug for ChannelOrdering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelOrdering").finish()
    }
}