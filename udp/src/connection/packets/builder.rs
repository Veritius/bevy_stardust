use bytes::{Bytes, BytesMut};
use crate::plugin::PluginConfiguration;

use super::frames::{Frame, FrameQueue};

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
    scratch: BytesMut,
}