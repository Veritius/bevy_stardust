use crate::MAXIMUM_TRANSPORT_UNITS;

#[inline]
pub(crate) const fn pipe_for_channel(pipes: u8, channels: u32, channel: u32) -> u8 {
    let channels_per_pipe = channels / pipes as u32;
    (channel / channels_per_pipe) as u8
}

pub(super) struct Pipe {
    pub local: u16,
    pub remote: u16,
    pub bitfield: u32,
    unacked: [PipeQueueSlot; 33],
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            local: 0,
            remote: 0,
            bitfield: 0,
            unacked: [PipeQueueSlot::new(); 33],
        }
    }
}

#[derive(Clone, Copy)]
struct PipeQueueSlot(usize, [u8; MAXIMUM_TRANSPORT_UNITS]);

impl PipeQueueSlot {
    fn new() -> Self {
        Self(0, [0u8; MAXIMUM_TRANSPORT_UNITS])
    }

    fn push_front(&mut self, item: &[u8]) {
        todo!()
    }
}