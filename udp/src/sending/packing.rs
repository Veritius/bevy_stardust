use std::sync::MutexGuard;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{MAXIMUM_PACKET_LENGTH, established::UdpConnection};

pub(super) struct PackingConfig {
    pub use_short_ids: bool,
}

/// Tries to pack octet strings into as little packets as possible.
/// Currently uses the First-Fit bin packing algorithm.
pub(super) fn pack_strings_first_fit<'a>(
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
        let buffer = 'bins: {
            // Find a bin
            for bin in bins.iter_mut() {
                if (bin.len() + scratch_len) > bin.capacity() { continue }
                break 'bins bin; // This one's good
            }

            // Create a new bin
            let buffer = Vec::with_capacity(MAXIMUM_PACKET_LENGTH);
            bins.push(buffer);
            break 'bins bins.last_mut().unwrap();
        };

        todo!();
    }

    // Return our packed bins
    bins
}