use std::cell::Cell;
use bevy::prelude::Resource;
use bytes::Bytes;
use thread_local::ThreadLocal;
use tracing::trace_span;
use crate::plugin::PluginConfiguration;
use super::frames::{Frame, FrameQueue, FrameQueueIter};

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
    /// `scratch` is scratch memory that can be shared between runs.
    pub fn run(
        &mut self,
        budget: usize,
        max_size: usize,
        context: PacketBuilderContext,
        scratch: &mut PackingBuilderScratch,
    ) -> Vec<Bytes> {
        // Check the budget is enough to work with
        assert!(max_size >= MIN_MTU, "MTU was too small");

        // Record these here because they reset when drain is called.
        let overall_estimate = self.queue.total_est();
        let reliable_estimate = self.queue.reliable_est();
        let unreliable_estimate = self.queue.unreliable_est();

        // Record data for debugging
        let trace_span = trace_span!("Packing");
        let _entered = trace_span.enter();
        trace_span.record("budget", budget);
        trace_span.record("mtu", max_size);
        trace_span.record("queue_est_total", overall_estimate);
        trace_span.record("queue_est_rel", reliable_estimate);
        trace_span.record("queue_est_no_rel", unreliable_estimate);

        // Get an iterator of frames that need to be put into packets
        // Automatically sorts the queue by priority using Frame's Ord impl
        let frames = self.queue.iter();

        // Shared state data used by all packing functions
        let shared_context = PackFnSharedCtx {
            frames,
            overall_estimate,
            reliable_estimate,
            unreliable_estimate,
        };

        // Case matching to try and find an optimal configuration of bins.
        let ret = match (overall_estimate, reliable_estimate, unreliable_estimate) {
            // There is no data to be transmitted.
            // Purpose: early return.
            (0, _, _) => { return Vec::with_capacity(0) },

            // There is only reliable data to be transmitted.
            // Purpose: concise, specialised packets.
            (_, x, 0) if x > 0 => pack_special_reliable_only(shared_context),

            // There is only unreliable data to be transmitted.
            // Purpose: concise, specialised packets.
            (_, 0, x) if x > 0 => pack_special_unreliable_only(shared_context),

            // There is mostly reliable data to be transmitted.
            // (a, b, c) if b > c && b.abs_diff(c) > a / 3 => todo!(),

            // There is mostly unreliable data to be transmitted.
            // (a, b, c) if c > b && b.abs_diff(c) > a / 3 => todo!(),

            // Generic case. No special behavior.
            _ => pack_generic(shared_context),
        };

        // Return
        return ret;
    }

    pub fn put<'a>(
        &'a mut self,
        frame: Frame,
    ) {
        self.queue.push(frame);
    }
}

struct PackFnSharedCtx<'a> {
    frames: FrameQueueIter<'a>,
    overall_estimate: usize,
    reliable_estimate: usize,
    unreliable_estimate: usize,
}

fn pack_special_reliable_only(
    mut ctx: PackFnSharedCtx,
) -> Vec<Bytes> {
    todo!()
}

fn pack_special_unreliable_only(
    mut ctx: PackFnSharedCtx,
) -> Vec<Bytes> {
    todo!()
}

fn pack_generic(
    mut ctx: PackFnSharedCtx,
) -> Vec<Bytes> {
    todo!()
}

/// Static information about the application.
pub(crate) struct PacketBuilderContext<'a> {
    pub config: &'a PluginConfiguration,
}

/// Scratch memory that can be shared between runs of [`PacketBuilder`].
pub(crate) struct PackingBuilderScratch {
    scratch: Vec<u8>,
}

impl PackingBuilderScratch {
    pub fn no_alloc() -> Self {
        Self {
            scratch: Vec::with_capacity(0),
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct PackingBuilderScratchCells {
    cells: ThreadLocal<Cell<PackingBuilderScratch>>,
}

impl PackingBuilderScratchCells {
    pub(super) fn get_cell(&self) -> &Cell<PackingBuilderScratch> {
        self.cells.get_or(|| {
            Cell::new(PackingBuilderScratch {
                scratch: Vec::with_capacity(1024),
            })
        })
    }
}