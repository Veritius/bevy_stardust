use bitflags::bitflags;

pub(super) struct PacketHeader {
    pub flags: PacketHeaderFlags,
}

bitflags! {
    pub struct PacketHeaderFlags: u8 {
        const FIRSTPKT = 1 << 0;
        const RELIABLE = 1 << 1;
    }
}