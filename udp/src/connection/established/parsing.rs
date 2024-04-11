use bevy_stardust::prelude::*;
use unbytes::*;
use crate::{connection::reliability::ReliablePacketHeader, plugin::PluginConfiguration, sequences::SequenceId, varint::VarInt};
use super::packet::PacketHeader;

pub(super) struct PacketHeaderData {
    pub flags: PacketHeader,
    pub reliable: Option<ReliablePacketHeader>,
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

pub(super) struct ParsedFrame {
    pub ident: u32,
    pub order: Option<SequenceId>,
    pub length: usize,
}

impl ParsedFrame {
    pub fn parse(
        reader: &mut Reader,
        config: &PluginConfiguration,
        registry: &ChannelRegistry,
    ) -> Result<Self, FrameParseError> {
        // Read the identity value
        const MAX: u64 = 2u64.pow(32);
        let ident: u64 = VarInt::read(reader)?.into();
        if ident > VarInt::MAX { return Err(FrameParseError::InvalidIdent); }
        let ident = ident as u32;

        // Read an ordering value
        let order = {
            // Check if an ordering value is present for this identity
            let has_order = match ident {
                // Ident 0 is always ordered
                0 => true,

                // We have to search up the ordering for Stardust channels
                _ => {
                    let cid = ChannelId::from(ident.wrapping_sub(1) as u32);
                    if let Some(config) = registry.channel_config(cid) {
                        config.ordered != OrderingGuarantee::Unordered
                    } else {
                        return Err(FrameParseError::InvalidIdent);
                    }
                },
            };

            if has_order {
                Some(reader.read_u16()?.into())
            } else {
                None
            }
        };

        // Get the length of the message
        let length = usize::from(VarInt::read(reader)?);

        return Ok(Self { ident, order, length })
    }
}

pub(super) enum FrameParseError {
    EndOfInput,
    InvalidIdent,
}

impl From<EndOfInput> for FrameParseError {
    #[inline]
    fn from(_: EndOfInput) -> Self {
        Self::EndOfInput
    }
}