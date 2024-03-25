use bytes::Bytes;

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct PacketHeader(pub u8);

impl PacketHeader {
    pub const RELIABLE: u8 = 1;
}

#[derive(Debug)]
pub(super) struct Frame {
    pub flags: u32,
    pub ident: u32,
    pub bytes: Bytes,
}

impl Frame {
    pub const IS_RELIABLE: u32 = 1 << 0;
    pub const IS_ORDERED: u32 = 1 << 1;
}