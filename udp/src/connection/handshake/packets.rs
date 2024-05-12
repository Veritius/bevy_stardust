use bytes::BufMut;
use unbytes::{EndOfInput, Reader};
use super::HandshakeResponseCode;

pub(super) struct HandshakeRelData {
    pub seq_ident: u16,
    pub ack_ident: u16,
    pub ack_bits: u16,
}

impl HandshakeRelData {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            seq_ident: reader.read_u16()?,
            ack_ident: reader.read_u16()?,
            ack_bits: reader.read_u16()?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        writer.put_u16(self.seq_ident);
        writer.put_u16(self.ack_ident);
        writer.put_u16(self.ack_bits);
    }
}

pub(super) struct HandshakeVerData {
    pub identifier: u64,
    pub major: u32,
    pub minor: u32,
}

impl HandshakeVerData {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            identifier: reader.read_u64()?,
            major: reader.read_u32()?,
            minor: reader.read_u32()?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        writer.put_u64(self.identifier);
        writer.put_u32(self.major);
        writer.put_u32(self.minor);
    }
}

pub(super) struct InitiatorHelloPacket {
    pub tr_ver: HandshakeVerData,
    pub app_ver: HandshakeVerData,
}

impl InitiatorHelloPacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            tr_ver: HandshakeVerData::read(reader)?,
            app_ver: HandshakeVerData::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        self.tr_ver.write(writer);
        self.app_ver.write(writer);
    }
}

pub(super) struct ListenerResponsePacket {
    pub tr_ver: HandshakeVerData,
    pub app_ver: HandshakeVerData,
    pub response: HandshakeResponseCode,
}

impl ListenerResponsePacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            tr_ver: HandshakeVerData::read(reader)?,
            app_ver: HandshakeVerData::read(reader)?,
            response: reader.read_u16()?.into(),
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        debug_assert_ne!(self.response, HandshakeResponseCode::Unknown);

        self.tr_ver.write(writer);
        self.app_ver.write(writer);
        writer.put_u16(self.response as u16);
    }
}

pub(super) struct InitiatorResponsePacket {
    pub response: HandshakeResponseCode,
}

impl InitiatorResponsePacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            response: reader.read_u16()?.into(),
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        writer.put_u16(self.response as u16);
    }
}