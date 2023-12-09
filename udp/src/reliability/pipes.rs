use crate::MAXIMUM_TRANSPORT_UNITS;

pub(super) struct Pipes {
    count: usize,
    pipes: Vec<Pipe>,
}

pub(super) struct Pipe {
    local: u16,
    remote: u16,
    bitfield: u32,
    unacked: [[u8; MAXIMUM_TRANSPORT_UNITS]; 33],
}