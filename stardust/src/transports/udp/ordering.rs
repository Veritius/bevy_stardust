use std::{collections::BTreeMap, fmt::Debug};
use crate::prelude::ChannelId;

/// Ordering data for a single peer.
#[derive(Debug)]
pub(super) struct Ordering {
    /// Ordering data of connection management messages.
    /// Mainly used during the handshake.
    pub main: OrderingData,
    /// Ordering data of individual channels.
    channels: BTreeMap<ChannelId, OrderingData>
}

impl Default for Ordering {
    fn default() -> Self {
        Self {
            main: OrderingData::default(),
            channels: Default::default()
        }
    }
}

/// Ordering data for a single channel.
pub(super) struct OrderingData {
    
}

impl Default for OrderingData {
    fn default() -> Self {
        Self {
            
        }
    }
}

impl Debug for OrderingData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelOrdering").finish()
    }
}