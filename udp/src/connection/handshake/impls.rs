// A big space for impl blocks.

use bytes::BufMut;
use untrusted::*;
use crate::utils::IntegerFromByteSlice;
use super::packets::*;

impl HandshakePacketHeader {
    pub fn from_bytes(reader: &mut Reader) -> Result<Self, EndOfInput> {
        return Ok(Self {
            sequence: u16::from_byte_slice(reader)?,
        })
    }

    pub fn write_bytes(&self, buffer: &mut impl BufMut) {
        buffer.put_u16(self.sequence);
    }
}

impl HandshakePacket for ClientHelloPacket {
    fn from_reader(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        todo!()
    }

    fn write_bytes(&self, buffer: &mut impl BufMut) {
        self.header.write_bytes(buffer);
    }
}

impl HandshakePacket for ServerHelloPacket {
    fn from_reader(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        todo!()
    }

    fn write_bytes(&self, buffer: &mut impl BufMut) {
        todo!()
    }
}

impl HandshakePacket for ClientFinalisePacket {
    fn from_reader(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        todo!()
    }

    fn write_bytes(&self, buffer: &mut impl BufMut) {
        todo!()
    }
}