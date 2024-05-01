use bytes::Bytes;
use crate::connection::packets::frames::*;
use super::PackFnSharedCtx;

/// Simple, naive packing algorithm. Works okay on any incoming sets.
pub(super) fn pack_naive(
    mut ctx: PackFnSharedCtx,
) -> Vec<Bytes> {
    // Storage for our finished packets
    // We allocate 1 slot as if we reach this point we have at least 1 message.
    let mut finished = Vec::with_capacity(1);

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

    todo!();

    // Return all unread frames back into the queue
    // This occurs if we have a high message load that exceeds our budget
    // We need to do this to ensure we don't drop packets that get unlucky
    ctx.frames.finish(unreliable.drain(..).chain(reliable.drain(..)));

    // Return the set of finished packets.
    return finished;
}