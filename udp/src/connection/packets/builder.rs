use std::cell::Cell;
use bevy::prelude::Resource;
use bytes::Bytes;
use thread_local::ThreadLocal;
use crate::plugin::PluginConfiguration;
use super::frames::{Frame, FrameQueue};

/*
    Packets are created using the first-fit bin packing algorithm.
    However, since we also need to have a header for each packet,
    we allocate an additional N bytes (BIN_HDR_SCR_SIZE) to each buffer.
    By doing this, we don't need to reallocate or copy when the time
    comes to create a packet header, which speeds things up significantly.
*/

/// The amount of space allocated for a frame header.
pub const BIN_HDR_SCR_SIZE: usize = 32;

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
        // Get an iterator of frames that need to be put into packets
        // Automatically sorts the queue by priority using Frame's Ord impl
        let mut frames = self.queue.drain();

        todo!()
    }

    pub fn put<'a>(
        &'a mut self,
        frame: Frame,
    ) {
        self.queue.push(frame);
    }
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