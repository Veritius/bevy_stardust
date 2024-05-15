use bytes::BufMut;
use unbytes::{EndOfInput, Reader};
use super::HandshakeResponseCode;

pub(super) struct HandshakePacketHeader {
    pub seq_ident: u16,
}

impl HandshakePacketHeader {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            seq_ident: reader.read_u16()?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        writer.put_u16(self.seq_ident);
    }
}

pub(super) struct HandshakePacketAcks {
    pub ack_ident: u16,
    pub ack_memory: u16,
}

impl HandshakePacketAcks {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            ack_ident: reader.read_u16()?,
            ack_memory: reader.read_u16()?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        writer.put_u16(self.ack_ident);
        writer.put_u16(self.ack_memory);
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
    pub respcode: HandshakeResponseCode,
    pub tr_ver: HandshakeVerData,
    pub app_ver: HandshakeVerData,
}

impl InitiatorHelloPacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            respcode: reader.read_u16()?.into(),
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
    pub respcode: HandshakeResponseCode,
    pub tr_ver: HandshakeVerData,
    pub app_ver: HandshakeVerData,
    pub acks: HandshakePacketAcks,
}

impl ListenerResponsePacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            respcode: reader.read_u16()?.into(),
            tr_ver: HandshakeVerData::read(reader)?,
            app_ver: HandshakeVerData::read(reader)?,
            acks: HandshakePacketAcks::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        debug_assert_ne!(self.respcode, HandshakeResponseCode::Unknown);

        writer.put_u16(self.respcode as u16);
        self.tr_ver.write(writer);
        self.app_ver.write(writer);
        self.acks.write(writer);
    }
}

pub(super) struct InitiatorResponsePacket {
    pub respcode: HandshakeResponseCode,
    pub acks: HandshakePacketAcks,
}

impl InitiatorResponsePacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            respcode: reader.read_u16()?.into(),
            acks: HandshakePacketAcks::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        writer.put_u16(self.respcode as u16);
        self.acks.write(writer);
    }
}