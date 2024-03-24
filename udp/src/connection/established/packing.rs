use std::cell::Cell;
use bevy_ecs::system::Resource;
use bytes::BytesMut;
use thread_local::ThreadLocal;
use crate::packet::MTU_SIZE;
use super::frame::Frame;

const BYTE_SCRATCH_SIZE: usize = MTU_SIZE;
const FRAME_STORE_SIZE: usize = 256;
const BIN_STORE_SIZE: usize = 1;
const BIN_ALLOC_SIZE: usize = MTU_SIZE;

#[derive(Resource, Default)]
pub(crate) struct PackingScratch(ThreadLocal<Cell<PackingScratchInner>>);

impl PackingScratch {
    pub(super) fn get_inner(&self) -> &Cell<PackingScratchInner> {
        self.0.get_or(|| {
            Cell::new(PackingScratchInner {
                bytes: BytesMut::with_capacity(BYTE_SCRATCH_SIZE),
                frames: Vec::with_capacity(FRAME_STORE_SIZE),
                bins: Vec::with_capacity(BIN_STORE_SIZE),
            })
        })
    }
}

pub(super) struct PackingScratchInner {
    bytes: BytesMut,
    frames: Vec<Frame>,
    bins: Vec<Bin>,
}

impl PackingScratchInner {
    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame)
    }
}

pub(super) struct PackingManager<'a> {
    scratch: &'a mut PackingScratchInner,
}

struct Bin {
    reliable: bool,
    data: Vec<u8>,
}