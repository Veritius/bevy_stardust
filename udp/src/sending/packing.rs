use bevy::prelude::*;
use bevy_stardust::prelude::*;

use crate::MAXIMUM_PACKET_LENGTH;

/// Tries to pack octet strings into as little packets as possible.
/// Currently uses the First-Fit bin packing algorithm.
pub(super) fn pack_strings<'a>(
    items: impl Iterator<Item = (ChannelId, Entity, &'a OctetString)>,
) -> Vec<Vec<u8>> {
    // Buffers for a simple algorithm to try and pack as many octet strings into one packet as possible
    // The algorithm just finds the first 'buffer' and writes the data to that, adding more buffers as necessary.
    let mut buffers: Vec<Vec<u8>> = vec![];

    // Pack all strings into packets
    for (channel, _, string) in items {
        // Check the string isn't too long since fragmenting isn't supported
        if string.len() > (MAXIMUM_PACKET_LENGTH - 20) {
            panic!("A sent octet string was too long ({} bytes). Fragmenting isn't supported right now, so it couldn't be sent.", string.len());
        }

        // Find or create a buffer that can store our string
        let buffer = 'buffer_find: {
            // Find a buffer
            for buffer in buffers.iter_mut() {
                // if (buffer.len() + scratch_len) > buffer.capacity() { continue }
                break 'buffer_find buffer; // This one's good
            }

            // Create a new buffer
            let buffer = Vec::with_capacity(MAXIMUM_PACKET_LENGTH);
            buffers.push(buffer);
            break 'buffer_find buffers.last_mut().unwrap();
        };

        todo!();
    }

    buffers
}