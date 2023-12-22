use crate::MAXIMUM_TRANSPORT_UNITS;

pub(super) struct Pipe {
    local: u16,
    remote: u16,
    bitfield: u32,
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
}