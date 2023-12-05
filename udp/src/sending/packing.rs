use std::sync::MutexGuard;
use bevy::prelude::{Entity, Mut};
use bevy_stardust::prelude::*;
use crate::{MAXIMUM_PACKET_LENGTH, established::UdpConnection};

pub(super) struct PackingConfig {
    pub use_short_ids: bool,
}

/// Tries to pack octet strings into as little packets as possible.
pub(super) fn pack_strings<'a>(
    packing_config: &PackingConfig,
    peer_data: &mut MutexGuard<(Mut<NetworkPeer>, Mut<UdpConnection>)>,
    items: impl Iterator<Item = (ChannelId, Entity, &'a OctetString)>,
) -> Vec<Vec<u8>> {
    // Bins for packing
    let mut bins: Vec<Vec<u8>> = vec![];

    // Scratch space
    let mut scratch_buf = [0u8; 1450];
    let mut scratch_len: usize = 0;

    // Pack all strings into packets
    for (channel, _, string) in items {
        // Check the string isn't too long since fragmenting isn't supported
        if string.len() > (MAXIMUM_PACKET_LENGTH - 20) {
            panic!("A sent octet string was too long ({} bytes). Fragmenting isn't supported right now, so it couldn't be sent.", string.len());
        }

        // Find or create a bin that can store our string
        // Uses the best-fit bin packing algorithm
        // https://en.wikipedia.org/wiki/Best-fit_bin_packing
        let bin = 'bins: {
            // Try to find the most suitable bin (least space remaining)
            let mut most_suitable: (usize, usize) = (usize::MAX, MAXIMUM_PACKET_LENGTH);
            for (index, bin) in bins.iter().enumerate() {
                let bin_space = bin.capacity() - bin.len();
                if scratch_len > bin_space { continue } // Check this bin has space for our message
                if bin_space < most_suitable.1 { continue } // Check if this bin has less space
                most_suitable = (index, bin_space);
            }

            // If none of the bins were suitable, create a new one and break
            if most_suitable.0 == usize::MAX {
                bins.push(Vec::with_capacity(MAXIMUM_PACKET_LENGTH));
                let ind = bins.len();
                break 'bins &mut bins[ind];
            }

            // Break with the most suitable bin
            break 'bins &mut bins[most_suitable.0];
        };

        todo!();
    }

    // Return our packed bins
    bins
}