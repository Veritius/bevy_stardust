use std::cell::Cell;
use bevy_ecs::system::Resource;
use bytes::BytesMut;
use thread_local::ThreadLocal;
use crate::{packet::MTU_SIZE, plugin::PluginConfiguration};
use super::frame::Frame;

const BYTE_SCRATCH_SIZE: usize = MTU_SIZE;
const FRAME_STORE_SIZE: usize = 256;
const BIN_STORE_SIZE: usize = 1;
const BIN_ALLOC_SIZE: usize = MTU_SIZE;

#[derive(Resource, Default)]
pub(crate) struct PackingScratch(ThreadLocal<Cell<PackingScratchData>>);

impl PackingScratch {
    pub(super) fn cell(&self) -> &Cell<PackingScratchData> {
        self.0.get_or(|| {
            Cell::new(PackingScratchData {
                bytes: BytesMut::with_capacity(BYTE_SCRATCH_SIZE),
                frames: Vec::with_capacity(FRAME_STORE_SIZE),
                bins: Vec::with_capacity(BIN_STORE_SIZE),
            })
        })
    }
}

pub(super) struct PackingScratchData {
    bytes: BytesMut,
    frames: Vec<Frame>,
    bins: Vec<Bin>,
}

impl PackingScratchData {
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

pub(super) struct PackingManager<'a> {
    scratch: &'a mut PackingScratchData,
    config: &'a PluginConfiguration,
}

impl<'a> PackingManager<'a> {
    pub fn build(
        scratch: &'a mut PackingScratchData,
        config: &'a PluginConfiguration,
    ) -> Self {
        Self { scratch, config }
    }

    pub fn run(&mut self) {
        // Record some data for debugging
        let trace_span = tracing::trace_span!("Packing packets");
        let _entered = trace_span.enter();
        trace_span.record("frames", self.scratch.frames.len());

        todo!()
    }
}

struct Bin {
    reliable: bool,
    data: Vec<u8>,
}