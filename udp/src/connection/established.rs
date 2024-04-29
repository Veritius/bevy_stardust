use super::packets::{builder::PacketBuilder, reader::PacketReader};

pub(super) struct Established {
    builder: PacketBuilder,
    reader: PacketReader,
}