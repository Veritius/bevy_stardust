use bytes::BufMut;
use unbytes::{EndOfInput, Reader};
use crate::appdata::NetworkVersionData;

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

pub(super) struct InitiatorHelloPacket {
    pub tr_ver: NetworkVersionData,
    pub app_ver: NetworkVersionData,
}

impl InitiatorHelloPacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            tr_ver: NetworkVersionData::from_bytes(reader)?,
            app_ver: NetworkVersionData::from_bytes(reader)?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        writer.put(&self.tr_ver.to_bytes()[..]);
        writer.put(&self.app_ver.to_bytes()[..]);
    }
}

pub(super) struct ListenerHelloPacket {
    pub tr_ver: NetworkVersionData,
    pub app_ver: NetworkVersionData,
    pub acks: HandshakePacketAcks,
}

impl ListenerHelloPacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            tr_ver: NetworkVersionData::from_bytes(reader)?,
            app_ver: NetworkVersionData::from_bytes(reader)?,
            acks: HandshakePacketAcks::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        writer.put(&self.tr_ver.to_bytes()[..]);
        writer.put(&self.app_ver.to_bytes()[..]);
        self.acks.write(writer);
    }
}

pub(super) struct InitiatorResponsePacket {
    pub acks: HandshakePacketAcks,
}

impl InitiatorResponsePacket {
    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(Self {
            acks: HandshakePacketAcks::read(reader)?,
        })
    }

    pub fn write(&self, writer: &mut impl BufMut) {
        self.acks.write(writer);
    }
}