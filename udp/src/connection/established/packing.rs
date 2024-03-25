use std::{cell::Cell, cmp::Ordering};
use bevy_ecs::system::Resource;
use bevy_stardust::channels::ChannelRegistryInner;
use bytes::BytesMut;
use thread_local::ThreadLocal;
use crate::{connection::established::frame::FrameFlags, packet::MTU_SIZE, plugin::PluginConfiguration};
use super::frame::Frame;

const BYTE_SCRATCH_SIZE: usize = MTU_SIZE;
const FRAME_STORE_SIZE: usize = 256;
const BIN_STORE_SIZE: usize = 1;
const BIN_ALLOC_SIZE: usize = MTU_SIZE;

#[derive(Resource, Default)]
pub(crate) struct PackingScratchCells(ThreadLocal<Cell<PackingScratch>>);

impl PackingScratchCells {
    pub(super) fn cell(&self) -> &Cell<PackingScratch> {
        self.0.get_or(|| {
            Cell::new(PackingScratch {
                bytes: BytesMut::with_capacity(BYTE_SCRATCH_SIZE),
                frames: Vec::with_capacity(FRAME_STORE_SIZE),
                bins: Vec::with_capacity(BIN_STORE_SIZE),
            })
        })
    }
}

pub(super) struct PackingScratch {
    bytes: BytesMut,
    frames: Vec<Frame>,
    bins: Vec<Bin>,
}

impl PackingScratch {
    pub fn empty() -> Self {
        Self {
            bytes: BytesMut::with_capacity(0),
            frames: Vec::with_capacity(0),
            bins: Vec::with_capacity(0),
        }
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame)
    }
}

#[derive(Clone, Copy)]
pub(super) struct PackingContext<'a> {
    pub config: &'a PluginConfiguration,
    pub registry: &'a ChannelRegistryInner,
}

pub(super) struct PackingInstance<'a> {
    scratch: &'a mut PackingScratch,
    context: PackingContext<'a>,
}

impl<'a> PackingInstance<'a> {
    pub fn build(
        scratch: &'a mut PackingScratch,
        context: PackingContext<'a>,
    ) -> Self {
        Self { scratch, context }
    }

    pub fn run(&mut self) {
        // Record some data for debugging
        let trace_span = tracing::trace_span!("Packing frames");
        let _entered = trace_span.enter();
        trace_span.record("frames", self.scratch.frames.len());

        // Sort the data to read reliable frames first
        let trace_span = tracing::trace_span!("Sorting frames");
        trace_span.in_scope(|| {
            self.scratch.frames.sort_unstable_by(Self::sort_frames)
        });

        todo!()
    }

    fn sort_frames(a: &Frame, b: &Frame) -> Ordering {
        match (
            (a.flags & FrameFlags::RELIABLE).0 > 0,
            (b.flags & FrameFlags::RELIABLE).0 > 0,
        ) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            _ => Ordering::Equal,
        }
    }
}

struct Bin {
    reliable: bool,
    data: Vec<u8>,
}