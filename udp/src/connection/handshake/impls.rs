// A big space for impl blocks.

use bytes::{BufMut, BytesMut};
use untrusted::*;
use crate::{connection::reliability::ReliablePacketHeader, utils::{slice_to_array, IntegerFromByteSlice}, appdata::NetworkVersionData};
use super::packets::*;

impl ClientHelloPacket {
    pub fn from_bytes(
        reader: &mut Reader,
    ) -> Result<Self, EndOfInput> {
        Ok(Self {
            transport: NetworkVersionData::from_bytes(slice_to_array::<16>(reader)?),
            application: NetworkVersionData::from_bytes(slice_to_array::<16>(reader)?),
            sequence_identifier: u16::from_byte_slice(reader)?,
        })
    }

    pub fn write_bytes(&self, buf: &mut BytesMut) {
        buf.put(&self.transport.to_bytes()[..]);
        buf.put(&self.application.to_bytes()[..]);
        buf.put_u16(self.sequence_identifier);
    }
}

impl ServerHelloPacket {
    pub fn from_bytes(
        reader: &mut Reader,
    ) -> Result<Self, EndOfInput> {
        let transport = NetworkVersionData::from_bytes(slice_to_array::<16>(reader)?);
        let application = NetworkVersionData::from_bytes(slice_to_array::<16>(reader)?);

        let sequence = u16::from_byte_slice(reader)?;
        let ack = u16::from_byte_slice(reader)?;
        let mut bfb = [0u8; 16];
        bfb[..2].copy_from_slice(reader.read_bytes(2)?.as_slice_less_safe());
        let ack_bitfield = u128::from_be_bytes(bfb);

        Ok(Self {
            transport,
            application,
            reliability: ReliablePacketHeader {
                sequence,
                ack,
                ack_bitfield,
            },
        })
    }

    pub fn write_bytes(&self, buf: &mut BytesMut) {
        buf.put(&self.transport.to_bytes()[..]);
        buf.put(&self.application.to_bytes()[..]);
        buf.put_u16(self.reliability.sequence);
        buf.put_u16(self.reliability.ack);
        buf.put(&self.reliability.ack_bitfield.to_be_bytes()[..2]);
    }
}

impl ClientFinalisePacket {
    pub fn from_bytes(
        reader: &mut Reader,
    ) -> Result<Self, EndOfInput> {
        let sequence = u16::from_byte_slice(reader)?;
        let ack = u16::from_byte_slice(reader)?;
        let mut bfb = [0u8; 16];
        bfb[..2].copy_from_slice(reader.read_bytes(2)?.as_slice_less_safe());
        let ack_bitfield = u128::from_be_bytes(bfb);

        Ok(Self {
            reliability: ReliablePacketHeader {
                sequence,
                ack,
                ack_bitfield,
            }
        })
    }

    pub fn write_bytes(&self, buf: &mut BytesMut) {
        buf.put_u16(self.reliability.sequence);
        buf.put_u16(self.reliability.ack);
        buf.put(&self.reliability.ack_bitfield.to_be_bytes()[..2]);
    }
}