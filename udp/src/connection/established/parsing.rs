use unbytes::*;
use crate::{connection::reliability::ReliablePacketHeader, plugin::PluginConfiguration};
use super::packet::PacketHeader;

pub(super) struct PacketHeaderData {
    flags: PacketHeader,
    reliable: Option<ReliablePacketHeader>,
}

impl PacketHeaderData {
    pub fn parse(
        reader: &mut Reader,
        config: &PluginConfiguration,
    ) -> Result<Self, EndOfInput> {
        let flags = PacketHeader(reader.read_byte()?);

        // Collect various information about the packet from the header
        let is_reliable = flags.0 & PacketHeader::RELIABLE > 0;

        // Get the reliable header
        let reliable = match is_reliable {
            false => None,
            true => {
                let seq = reader.read_u16()?.into();
                let ack = reader.read_u16()?.into();

                let mut bytes = [0u8; 16];
                let bits_slice = reader.read_slice(config.reliable_bitfield_length)?;
                bytes[..config.reliable_bitfield_length].copy_from_slice(bits_slice);
                let bits = u128::from_be_bytes(bytes);

                Some(ReliablePacketHeader { seq, ack, bits })
            }
        };

        Ok(Self { flags, reliable })
    }
}