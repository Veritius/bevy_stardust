mod naive;

use bytes::Bytes;
use tracing::trace_span;
use crate::{connection::reliability::ReliabilityState, plugin::PluginConfiguration};
use super::frames::{FrameQueue, FrameQueueIter, FrameQueueStats, SendFrame};

/*
    Packets are created using the first-fit bin packing algorithm.
    However, since we also need to have a header for each packet,
    we allocate an additional N bytes (BIN_HDR_SCR_SIZE) to each buffer.
    By doing this, we don't need to reallocate or copy when the time
    comes to create a packet header, which speeds things up significantly.
*/

/// The minimum MTU for a connection.
/// Values lower than this will panic.
pub(crate) const MIN_MTU: usize = 128;

/// The amount of space allocated for a frame header.
const BIN_HDR_SCR_SIZE: usize = 32;

pub(crate) struct PacketBuilder {
    queue: FrameQueue,
}

impl Default for PacketBuilder {
    fn default() -> Self {
        Self {
            queue: FrameQueue::with_capacity(32),
        }
    }
}

impl PacketBuilder {
    /// Runs the packet builder.
    /// 
    /// `budget` is the max total bytes that are returned for sending.
    /// `max_size` is the maximum size of an individual packet.
    /// 
    /// `context` is information about the application.
    pub fn run(
        &mut self,
        budget: usize,
        max_size: usize,
        context: PacketBuilderContext,
    ) -> Vec<Bytes> {
        // Check the budget is enough to work with
        assert!(max_size >= MIN_MTU, "MTU was too small");

        // Record these here because they reset when drain is called.
        let queue_stats = self.queue.assess();

        // Record data for debugging
        let trace_span = trace_span!("Packing");
        let _entered = trace_span.enter();
        trace_span.record("budget", budget);
        trace_span.record("mtu", max_size);
        trace_span.record("total_bytes", queue_stats.total_bytes_estimate);
        trace_span.record("total_frames", queue_stats.total_frames_count);
        trace_span.record("unreliable_frames", queue_stats.unreliable_frames_count);
        trace_span.record("reliable_frames", queue_stats.reliable_frames_count);

        // Get an iterator of frames that need to be put into packets
        // Automatically sorts the queue by priority using Frame's Ord impl
        let frames = self.queue.iter();

        // Shared state data used by all packing functions
        let shared_context = PackFnSharedCtx {
            context,
            frames,
            budget,
            max_size,
            stats: queue_stats.clone(),
        };

        // Case matching to try and find an optimal configuration of bins.
        let ret = match (
            queue_stats.total_frames_count,
            queue_stats.unreliable_frames_count,
            queue_stats.reliable_frames_count,
        ) {
            // There is no data to be transmitted.
            // Purpose: early return.
            (0, _, _) => { return Vec::with_capacity(0) },

            // There is mostly reliable data to be transmitted.
            // (a, b, c) if b > c && b.abs_diff(c) > a / 3 => todo!(),

            // There is mostly unreliable data to be transmitted.
            // (a, b, c) if c > b && b.abs_diff(c) > a / 3 => todo!(),

            // Generic case. No special behavior.
            _ => naive::pack_naive(shared_context),
        };

        // Return
        return ret;
    }

    pub fn put<'a>(
        &'a mut self,
        frame: SendFrame,
    ) {
        self.queue.push(frame);
    }
}

/// Static information about the application.
pub(crate) struct PacketBuilderContext<'a> {
    pub config: &'a PluginConfiguration,
    pub rel_state: &'a mut ReliabilityState,
    pub scratch: &'a mut Vec<u8>,
}

struct PackFnSharedCtx<'a> {
    context: PacketBuilderContext<'a>,
    frames: FrameQueueIter<'a>,
    budget: usize,
    max_size: usize,
    stats: FrameQueueStats,
}