use bevy::prelude::Entity;
use bevy_stardust::prelude::*;
use crate::{MAXIMUM_TRANSPORT_UNITS, established::UdpConnection};

pub(super) struct PackingConfig {
    pub use_short_ids: bool,
}

/// Tries to pack octet strings into as little packets as possible.
pub(super) fn pack_strings<'a>(
    packing_config: &PackingConfig,
    channels: &ChannelRegistry,
    peer_data: &mut UdpConnection,
    strings: impl Iterator<Item = (ChannelId, Entity, &'a OctetString)>,
) -> Vec<Vec<u8>> {
    // Reliable and unreliable bins
    let mut reliable_bins: Vec<Vec<u8>> = vec![];
    let mut unreliable_bins: Vec<Vec<u8>> = vec![];

    // Working space
    let mut scratch = [0u8; MAXIMUM_TRANSPORT_UNITS];
    let mut scratch_len: usize = 0;

    // Iterate all strings and pack them
    for (channel, _, string) in strings {
        // Information about the channel
        let data = channels.get_from_id(channel).unwrap();

        // Write data
        scratch[0..3].clone_from_slice(&Into::<[u8;3]>::into(channel));
        scratch_len += 3;

        // Ordering data
        if data.ordered {
            todo!()
        }

        // Push to the set
        match data.reliable {
            true => pack_best_fit(|| {
                Vec::with_capacity(MAXIMUM_TRANSPORT_UNITS)
            }, &scratch[..scratch_len], &mut reliable_bins),
            false => pack_best_fit(|| {
                Vec::with_capacity(MAXIMUM_TRANSPORT_UNITS)
            }, &scratch[..scratch_len], &mut unreliable_bins),
        }

        // Clean up for the next string
        scratch_len = 0;
    }

    // Return both bins
    reliable_bins.drain(..)
    .chain(unreliable_bins.drain(..))
    .collect::<Vec<_>>()
}

fn pack_best_fit(
    mut new_bin: impl FnMut() -> Vec<u8>,
    data: &[u8],
    bins: &mut Vec<Vec<u8>>
) {
    let length = data.len();

    // Pick the most suitable bin
    let mut most_suitable = (usize::MAX, usize::MAX);
    for (index, bin) in bins.iter().enumerate() {
        let remaining_space = bin.capacity().saturating_sub(bin.len());
        if remaining_space < length { continue }
        if remaining_space > most_suitable.1 { continue }
        most_suitable = (index, remaining_space)
    }

    // Get or create the bin
    let bin = match most_suitable.0 {
        usize::MAX => {
            bins.push(new_bin());
            let len = bins.len();
            &mut bins[len]
        },
        _ => &mut bins[most_suitable.0]
    };

    // Write to the bin
    todo!()
}