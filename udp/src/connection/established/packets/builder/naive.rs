use std::time::Instant;

use bytes::{Bytes, BufMut};
use crate::{connection::{established::packets::header::PacketHeaderFlags, reliability::UnackedPacket}, varint::VarInt};
use super::PackFnSharedCtx;

/// The amount of space allocated for a frame header.
const BIN_HDR_SCR_SIZE: usize = 32;

/// For every reliable frame, this many unreliable frames will be sent.
const UNRELIABLE_FRAME_BIAS: usize = 3;

// The relative amount of data that can be wasted in exchange for avoiding a reallocation.
// If this threshold is exceeded, bins will reallocate when turning into Bytes.
// These two values apply to reliable and unreliable bins, respectively.
const RELIABLE_WASTE_TOLERANCE: usize = 2;
const UNRELIABLE_WASTE_TOLERANCE: usize = 4;

/// Simple, naive packing algorithm. Works okay on any incoming sets.
pub(super) fn pack_naive(
    mut ctx: PackFnSharedCtx,
) -> Vec<Bytes> {
    // Storage for bins that are still being packed.
    let mut bins: Vec<WorkingBin> = Vec::with_capacity(1);

    // Sort frames into new vecs based on whether they're reliable or not
    // This maintains the priority order as frames are iterated in order
    let mut unreliable = Vec::with_capacity(ctx.stats.unreliable_frames_count);
    let mut reliable = Vec::with_capacity(ctx.stats.reliable_frames_count);
    while let Some(frame) = ctx.frames.next() {
        match frame.reliable {
            false => unreliable.push(frame),
            true => reliable.push(frame),
        }
    }

    // Try to pack as many frames as possible
    let mut used = 0;
    let mut idx = 0;
    'outer: loop {
        // Determine the next packet to read
        let rel_frm = {
            let rel_is_empty = reliable.is_empty();
            let unrel_is_empty = unreliable.is_empty();

            match (idx % UNRELIABLE_FRAME_BIAS, rel_is_empty, unrel_is_empty) {
                (_, true, true) => { break 'outer },
                (_, true, false) => false,
                (_, false, true) => true,
                (x, _, _) if x == 0 => true,
                (_, _, _) => false,
            }
        };

        // Get the frame from either buffer
        // Unwraps are fine since we checked previously
        let frame = match rel_frm {
            true => reliable.pop().unwrap(),
            false => unreliable.pop().unwrap(),
        };

        // Store and compare the estimate to make sure
        // that we don't go over budget, and for later use
        let frame_size_estimate = frame.bytes_est();
        if (frame_size_estimate + used) > ctx.budget { continue 'outer; }

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
            // Also fill it with a certain amount of bytes for a 'dead' header
            let mut bin = WorkingBin::new(rel_frm, ctx.max_size);
            bin.inner_data.extend((0..BIN_HDR_SCR_SIZE).into_iter().map(|_| 0));
            bins.push(bin);
            let bin_idx = bins.len();
            break 'bin &mut bins[bin_idx - 1];
        };

        let scr = &mut bin.inner_data;

        #[cfg(debug_assertions)]
        let previous_length = scr.len();

        // Put the frame flags into the bin
        scr.put_u8(frame.flags.into());

        // Put the frame type into the bin
        scr.put_u8(frame.ftype.into());

        // If the frame has an identifier, put it in.
        if let Some(ident) = frame.ident {
            ident.write(scr);
        }

        // If the frame has an ordering, put it in!
        if let Some(order) = frame.order {
            scr.put_u16(order.into());
        }

        // Put the payload length into the bin using a varint
        // Unwrapping is fine since I doubt anyone will try to
        // send a payload with a length of 4,611 petabytes.
        // Not that there's any computers that can even store that.
        VarInt::try_from(frame.payload.len()).unwrap().write(scr);

        // Put in the payload itself
        scr.put(frame.payload);

        #[cfg(debug_assertions)]
        assert_eq!(scr.len() - previous_length - 1, frame_size_estimate);

        // Update the bin
        bin.is_reliable |= rel_frm;

        // Increment the frame index
        idx += 1;
        used += frame_size_estimate;
    }

    // Return all unread frames back into the queue
    // This occurs if we have a high message load that exceeds our budget
    // We need to do this to ensure we don't drop packets that get unlucky
    // This can be done at any point, so we might as well do it here.
    ctx.frames.finish(unreliable.drain(..).chain(reliable.drain(..)));

    // Take all bins, and generate their headers
    // If a bin is reliable, this is where it's recorded
    // Bins are also frozen into a Bytes for returning
    bins.drain(..)
    .map(|mut bin| {
        // Get and clear the scratch buffer for use
        let scr = &mut ctx.context.scratch;
        scr.clear();

        // Create the packet header flags and push them
        let mut header = PacketHeaderFlags::EMPTY;
        if bin.is_reliable { header |= PacketHeaderFlags::RELIABLE }
        scr.put_u8(header.0);

        // Get the current reliability state
        // If this is reliable, push a sequence id for this packet
        let rel_hdr = ctx.context.rel_state.clone();
        if bin.is_reliable {
            scr.put_u16(rel_hdr.local_sequence.0);
            ctx.context.rel_state.advance();
        }

        // Put acknowledgement data
        // Unlike the sequence id, this is always present
        scr.put_u16(rel_hdr.remote_sequence.0);
        let bf_bts = rel_hdr.ack_memory.into_array();
        scr.put(&bf_bts[..ctx.context.config.reliable_bitfield_length]);

        // It's very important we don't accidentally overrun
        // the 'dead header' we've been assigned. We check here just in case.
        // Though, it'll probably panic anyway, but hey, it's easier to debug.
        debug_assert!(scr.len() <= BIN_HDR_SCR_SIZE);

        // Put the new header into the unused space in the bin's allocation
        // This makes sure the end of the header slice lines up with the start
        // of the payload, which we generated earlier, and put in this buffer.
        let hdr_len = scr.len();
        let offset = BIN_HDR_SCR_SIZE - hdr_len;
        bin.inner_data[offset..BIN_HDR_SCR_SIZE].clone_from_slice(&scr);

        // Calculate how much space has been wasted or otherwise unused in the allocation.
        // We also pick a threshold value based on whether or not the bin is reliable.
        // If a certain ratio of bytes in the allocation are unused, this triggers a reallocation.
        let utilised_bytes = bin.inner_data.len() - offset;
        let unused_bytes = bin.inner_data.capacity() - utilised_bytes;
        let usage_ratio = utilised_bytes / unused_bytes;
        let tolerance = match bin.is_reliable {
            true => RELIABLE_WASTE_TOLERANCE,
            false => UNRELIABLE_WASTE_TOLERANCE,
        };

        // Decide whether it's worth reallocating
        // This is what creates the Bytes we return
        let bytes = match usage_ratio > tolerance {
            true => {
                // Case 1: reallocate data
                // We don't really need to do anything, since the 
                // copy_from_slice fn does everything for us.
                Bytes::copy_from_slice(&bin.inner_data[offset..])
            },
            false => { 
                // Case 2: don't reallocate
                // This avoids reallocation by filling the vec
                // up to its length with data we don't care about.
                // Since Bytes reallocates based on length, this
                // prevents the reallocation.
                let len = bin.inner_data.len();
                bin.inner_data[len..].fill(0);
                let bytes = Bytes::from(bin.inner_data);
                bytes.slice(offset..len)
            },
        };

        // If the bin is reliable, record the payload
        // This makes sure we can resend it on failure
        if bin.is_reliable {
            // Excludes the header from the slice
            let payload = bytes.slice(hdr_len..);
            ctx.context.rel_packets.insert(rel_hdr.local_sequence, UnackedPacket {
                payload,
                time: Instant::now(),
            });
        }

        // Return the full bytes object
        return bytes;
    })
    .collect::<Vec<Bytes>>()
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