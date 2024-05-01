use bytes::{Bytes, BufMut};
use crate::{connection::packets::frames::*, varint::VarInt};
use super::PackFnSharedCtx;

/// For every reliable frame, this many unreliable frames will be sent.
const UNRELIABLE_FRAME_BIAS: usize = 3;

/// Simple, naive packing algorithm. Works okay on any incoming sets.
pub(super) fn pack_naive(
    mut ctx: PackFnSharedCtx,
) -> Vec<Bytes> {
    // Storage for our finished packets
    // We allocate 1 slot as if we reach this point we have at least 1 message.
    let mut finished = Vec::with_capacity(1);

    // Storage for bins that are still being packed.
    let mut bins: Vec<WorkingBin> = Vec::with_capacity(1);

    // Sort frames into new vecs based on whether they're reliable or not
    // This maintains the priority order as frames are iterated in order
    let mut unreliable = Vec::with_capacity(ctx.stats.unreliable_frames_count);
    let mut reliable = Vec::with_capacity(ctx.stats.reliable_frames_count);
    while let Some(frame) = ctx.frames.next() {
        match frame.flags.any_high(FrameFlags::RELIABLE) {
            false => unreliable.push(frame),
            true => reliable.push(frame),
        }
    }

    // Try to pack as many frames as possible
    let mut used = 0;
    let mut idx = 0;
    loop {
        // Check if we've run out of frames to pack
        if reliable.is_empty() && unreliable.is_empty() { break }

        // Store whether or not the current frame will be reliable
        let rel_frm = !(idx % UNRELIABLE_FRAME_BIAS == 0 || reliable.is_empty());

        // Get the frame from either buffer
        // Unwraps are fine since we checked previously
        let frame = match rel_frm {
            true => reliable.pop().unwrap(),
            false => unreliable.pop().unwrap(),
        };

        // Store and compare the estimate to make sure
        // that we don't go over budget, and for later use
        let frame_size_estimate = frame.bytes_est();
        if (frame_size_estimate + used) > ctx.budget { continue }

        // Find a suitable bin to pack into
        let bin = 'bin: {
            // Modified first-fit packing algorithm
            let pk = bins
                .iter()
                .enumerate()
                .find(|(_, bin)| {
                    if bin.remaining() < frame_size_estimate { return false };
                    if rel_frm != bin.is_reliable { return false };
                    true
                })
                .map(|(idx, _)| idx);

            // First fit found a suitable bin
            if let Some(idx) = pk {
                break 'bin &mut bins[idx];
            }

            // Create a new bin for use
            bins.push(WorkingBin::new(rel_frm, ctx.max_size));
            let bin_idx = bins.len();
            break 'bin &mut bins[bin_idx];
        };

        let scr = &mut bin.inner_data;

        #[cfg(debug_assertions)]
        let previous_length = scr.len();

        // Put the frame type into the bin
        scr.put_u8(frame.ftype.into());

        // Put the payload length into the bin using a varint
        // Unwrapping is fine since I doubt anyone will try to
        // send a payload with a length of 4,611 petabytes.
        // Not that there's any computers that can even store that.
        VarInt::try_from(frame.payload.len()).unwrap();

        // Put in the payload itself
        scr.put(frame.payload);

        #[cfg(debug_assertions)]
        assert_eq!(previous_length - scr.len(), frame_size_estimate);

        // Increment the frame index
        idx += 1;
        used += frame_size_estimate;
    }

    // Return all unread frames back into the queue
    // This occurs if we have a high message load that exceeds our budget
    // We need to do this to ensure we don't drop packets that get unlucky
    ctx.frames.finish(unreliable.drain(..).chain(reliable.drain(..)));

    // Return the set of finished packets.
    return finished;
}

struct WorkingBin {
    is_reliable: bool,
    inner_data: Vec<u8>,
}

impl WorkingBin {
    fn new(reliable: bool, mtu: usize) -> Self {
        Self {
            is_reliable: reliable,
            inner_data: Vec::with_capacity(mtu),
        }
    }

    #[inline]
    fn remaining(&self) -> usize {
        self.inner_data.capacity() - self.inner_data.len()
    }
}