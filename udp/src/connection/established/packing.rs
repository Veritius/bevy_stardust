use std::{cell::Cell, cmp::Ordering};
use bevy_ecs::system::Resource;
use bevy_stardust::channels::ChannelRegistryInner;
use bytes::{BufMut, Bytes, BytesMut};
use thread_local::ThreadLocal;
use crate::{connection::established::{frame::FrameFlags, packet::PacketHeader}, packet::MTU_SIZE, plugin::PluginConfiguration, varint::VarInt};
use super::{frame::Frame, Established};

const BYTE_SCRATCH_SIZE: usize = MTU_SIZE;
const FRAME_STORE_SIZE: usize = 256;
const BIN_STORE_SIZE: usize = 1;

const BIN_HDR_SIZE: usize = 32;
const BIN_PLD_SIZE: usize = MTU_SIZE;
const BIN_TTL_SIZE: usize = BIN_HDR_SIZE + BIN_PLD_SIZE;
const REL_DEAD_MAX: usize = 128;

#[derive(Resource, Default)]
pub(crate) struct PackingScratchCells(ThreadLocal<Cell<PackingScratch>>);

impl PackingScratchCells {
    pub(super) fn cell(&self) -> &Cell<PackingScratch> {
        self.0.get_or(|| {
            Cell::new(PackingScratch {
                bytes: BytesMut::with_capacity(BYTE_SCRATCH_SIZE),
                frames: Vec::with_capacity(FRAME_STORE_SIZE),
                bins: Vec::with_capacity(BIN_STORE_SIZE),
                fin: Vec::with_capacity(BIN_STORE_SIZE),
            })
        })
    }
}

pub(super) struct PackingScratch {
    bytes: BytesMut,
    frames: Vec<Frame>,
    bins: Vec<Bin>,
    fin: Vec<Finished>,
}

impl PackingScratch {
    pub fn empty() -> Self {
        Self {
            bytes: BytesMut::with_capacity(0),
            frames: Vec::with_capacity(0),
            bins: Vec::with_capacity(0),
            fin: Vec::with_capacity(0),
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
    component: &'a mut Established,
    scratch: &'a mut PackingScratch,
    context: PackingContext<'a>,
}

impl<'a> PackingInstance<'a> {
    pub fn build(
        component: &'a mut Established,
        scratch: &'a mut PackingScratch,
        context: PackingContext<'a>,
    ) -> Self {
        Self { component, scratch, context }
    }

    pub fn run(&'a mut self) -> impl Iterator<Item = Finished> + 'a {
        // Record some data for debugging
        let trace_span = tracing::trace_span!("Running instance");
        let _entered = trace_span.enter();
        trace_span.record("frames", self.scratch.frames.len());

        // Sort the data to read reliable frames first
        let trace_span = tracing::trace_span!("Sorting frames");
        trace_span.in_scope(|| {
            self.scratch.frames.sort_unstable_by(Self::sort_frames)
        });

        // Pack frames into bins
        let trace_span = tracing::trace_span!("Packing frames");
        trace_span.in_scope(|| {
            for frame in self.scratch.frames.drain(..) {
                Self::write_single_frame(&frame, &mut self.component, &mut self.scratch.bytes);
                let bin = Self::get_or_make_bin(&mut self.scratch.bins, self.scratch.bytes.len());
                bin.data.extend_from_slice(&self.scratch.bytes);
                self.scratch.bytes.clear();
            }
        });

        // Generate bin headers
        let trace_span = tracing::trace_span!("Finalizing packets");
        trace_span.in_scope(|| {
            for bin in self.scratch.bins.drain(..) {
                let fin = Self::gen_bin_header(bin, self.context, &mut self.scratch.bytes, &mut self.component);
                self.scratch.fin.push(fin);
            }
        });

        // Return a draining iterator over all finished packets
        return self.scratch.fin.drain(..)
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

    fn write_single_frame(
        frame: &Frame,
        component: &mut Established,
        scratch: &mut BytesMut,
    ) {
        // Message identifier
        VarInt::from(frame.ident).write(scratch);

        // Ordering data
        if (frame.flags & FrameFlags::ORDERED).0 > 0 {
            let seq = component.ordering(frame.ident).advance();
            scratch.put_u16(seq.into());
        }

        // Message length
        VarInt::try_from(frame.bytes.len()).unwrap().write(scratch);

        // Insert the payload
        scratch.put(&*frame.bytes);
    }

    fn get_or_make_bin(
        bins: &mut Vec<Bin>,
        size: usize,
    ) -> &mut Bin {
        // Returns the first bin that has enough space to store the data
        let found_bin = bins.iter_mut()
        .enumerate()
        .find(|(_, bin)| {
            (bin.data.capacity() - bin.data.len()) >= size
        })
        .map(|(index, _)| index);

        // Return the bin if Some
        if let Some(index) = found_bin {
            return &mut bins[index];
        }

        // Make a bin instead
        let bin = Bin::new();
        bins.push(bin);
        return bins.last_mut().unwrap();
    }

    fn gen_bin_header(
        mut bin: Bin,
        context: PackingContext,
        scratch: &mut BytesMut,
        component: &mut Established,
    ) -> Finished {
        // Packet header
        let mut hdr = PacketHeader::default();
        if bin.reliable { hdr.0 |= PacketHeader::RELIABLE; }
        scratch.put_u8(hdr.0);

        // This has to be out of scope of the next
        // block, so it can be used later on.
        let mut sequence = 0.into();

        // Reliability header
        if bin.reliable {
            let header = component.reliability.header();
            component.reliability.advance();

            scratch.put_u16(header.seq.into());
            scratch.put_u16(header.ack.into());

            let bytes = header.bits.to_be_bytes();
            scratch.put(&bytes[..context.config.reliable_bitfield_length]);

            sequence = header.seq;
        }

        // Copy our new header to the dead prefix in the bin
        let div = scratch.len();
        let offset = BIN_HDR_SIZE - div;
        bin.data[offset..BIN_HDR_SIZE].clone_from_slice(&scratch);
        let end = bin.data.len();

        // Fill the rest of the bin's unused space with zero bytes
        // This is necessary to prevent a reallocation when converting to Bytes
        // We slice the new Bytes object anyway so this data won't be sent
        // While this wastes some memory, it's dropped soon enough.
        // However, if the packet is reliable, it'll be kept around.
        // We impose a limit for this case so we don't waste too much memory.

        // Theoretically, it should be fine to (unsafely) extend the length
        // making the Bytes store uninitialised data. Since that section will
        // never be readable outside of this function, it could be slightly faster.

        let remaining = bin.data.capacity() - bin.data.len();
        if !bin.reliable && remaining > REL_DEAD_MAX {
            bin.data.extend((0..remaining).map(|_| { 0 }));
            debug_assert_eq!(bin.data.capacity(), bin.data.len());
        }

        // Convert to bytes
        let full = Bytes::from(bin.data).slice(..end);

        // Store in reliable
        if bin.reliable {
            component.reliability.record(sequence, full.slice(div..));
        }

        // Clean up and return finished
        scratch.clear();
        return Finished { full, div }
    }
}

struct Bin {
    reliable: bool,
    data: Vec<u8>,
}

impl Bin {
    fn new() -> Self {
        let mut data = Vec::with_capacity(BIN_TTL_SIZE);
        data.extend_from_slice(&[0u8; BIN_HDR_SIZE]);

        Self {
            reliable: bool::default(),
            data,
        }
    }
}

pub(super) struct Finished {
    full: Bytes,
    div: usize,
}

impl Finished {
    pub fn full(&self) -> Bytes {
        self.full.clone()
    }

    pub fn header(&self) -> Bytes {
        self.full.slice(..self.div)
    }

    pub fn payload(&self) -> Bytes {
        self.full.slice(self.div..)
    }
}