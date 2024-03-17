use crate::connection::reliability::{ReliabilityState, ReliablePackets};
use super::packing::PackingManager;

pub(super) struct River {
    pub river_id: u8,
    pub packer: PackingManager,
    pub reliability: ReliablePackets,
}

impl River {
    pub fn new(
        id: u8,
        pk_size: usize,
        rel_state: ReliabilityState,
    ) -> Self {
        Self {
            river_id: id,
            packer: PackingManager::new(pk_size),
            reliability: ReliablePackets::new(rel_state),
        }
    }
}