use std::sync::MutexGuard;
use bevy::prelude::{Entity, Mut};
use bevy_stardust::prelude::*;
use crate::{MAXIMUM_PACKET_LENGTH, established::UdpConnection};

pub(super) struct PackingConfig {
    pub use_short_ids: bool,
}

/// First-fit bin packing algorithm.
pub(super) fn first_fit(
    element: usize,
    bins: &mut dyn Iterator<Item = (usize, usize, usize)>,
) -> usize {
    for (index, capacity, length) in bins {
        if (length + element) > capacity { return index }
    }

    return usize::MAX
}

/// Tries to pack octet strings into as little packets as possible, depending on your choice of `packing_alg`
pub(super) fn pack_strings<'a>(
    packing_alg: impl Fn(usize, &mut dyn Iterator<Item = (usize, usize, usize)>) -> usize,
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
        let index = packing_alg(scratch_len, &mut bins.iter().enumerate().map(|(index, f)| (index, f.capacity(), f.len())));
        if index == usize::MAX {
            let new_bin = Vec::with_capacity(MAXIMUM_PACKET_LENGTH);
            bins.push(new_bin);
        }

        // Access the bin we'll be putting bytes into
        let working_bin = match index {
            usize::MAX => {
                let len = bins.len();
                &mut bins[len]
            },
            _ => &mut bins[index],
        };

        todo!();
    }

    // Return our packed bins
    bins
}