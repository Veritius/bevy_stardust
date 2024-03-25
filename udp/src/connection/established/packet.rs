#[derive(Debug, Default, Clone, Copy)]
pub(super) struct PacketHeader(pub u8);

impl PacketHeader {
    pub const RELIABLE: u8 = 1;
}