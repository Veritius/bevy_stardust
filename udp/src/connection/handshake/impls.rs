// A big space for impl blocks.

// TODO: When try_trait_v2 stabilises, this code can be massively simplified using the try operator (?)

use bytes::{Buf, BufMut};
use crate::{appdata::NetworkVersionData, connection::handshake::codes::HandshakeResponseCode, utils::array_from_slice};
use super::packets::*;

impl HandshakePacketHeader {
    pub fn from_bytes(buf: &mut impl Buf) -> Result<Self, ()> {
        return Ok(Self {
            sequence: buf.get_u16(),
        })
    }

    pub fn write_bytes(&self, buffer: &mut impl BufMut) {
        buffer.put_u16(self.sequence);
    }
}

impl ClosingPacket {
    pub fn write_bytes(&self, buffer: &mut impl BufMut) {
        self.header.write_bytes(buffer);
        buffer.put_u16(self.reason as u16);
        if let Some(additional) = &self.additional {
            buffer.put(&**additional)
        }
    }
}

impl HandshakePacket for ClientHelloPacket {
    fn from_slice<T: Buf>(buf: &mut T) -> HandshakeParsingResponse<Self> {
        let transport = NetworkVersionData::from_bytes(buf);
        let application = NetworkVersionData::from_bytes(buf);

        HandshakeParsingResponse::Continue(Self {
            transport,
            application,
        })
    }

    fn write_bytes(&self, buffer: &mut impl BufMut) {
        buffer.put(&self.transport.to_bytes()[..]);
        buffer.put(&self.application.to_bytes()[..]);
    }
}

impl HandshakePacket for ServerHelloPacket {
    fn from_slice<T: Buf>(buf: &mut T) -> HandshakeParsingResponse<Self> {
        // Check the response code
        let response = HandshakeResponseCode::from(buf.get_u16());
        if response != HandshakeResponseCode::Continue {
            return HandshakeParsingResponse::TheyRejected(response)
        }

        let transport = NetworkVersionData::from_bytes(buf);
        let application = NetworkVersionData::from_bytes(buf);

        let reliability_ack = buf.get_u16();
        let reliability_bits = buf.get_u16();

        HandshakeParsingResponse::Continue(Self {
            transport,
            application,
            reliability_ack,
            reliability_bits,
        })
    }

    fn write_bytes(&self, buffer: &mut impl BufMut) {
        // Write response code
        buffer.put_u16(HandshakeResponseCode::Continue as u16);

        buffer.put(&self.transport.to_bytes()[..]);
        buffer.put(&self.application.to_bytes()[..]);

        buffer.put_u16(self.reliability_ack);
        buffer.put_u16(self.reliability_bits);
    }
}

impl HandshakePacket for ClientFinalisePacket {
    fn from_slice<T: Buf>(buf: &mut T) -> HandshakeParsingResponse<Self> {
        // Check the response code
        let response = HandshakeResponseCode::from(buf.get_u16());
        if response != HandshakeResponseCode::Continue {
            return HandshakeParsingResponse::TheyRejected(response)
        }

        let reliability_ack = buf.get_u16();
        let reliability_bits = buf.get_u16();

        return HandshakeParsingResponse::Continue(Self {
            reliability_ack,
            reliability_bits,
        })
    }

    fn write_bytes(&self, buffer: &mut impl BufMut) {
        // Write response code
        buffer.put_u16(HandshakeResponseCode::Continue as u16);

        buffer.put_u16(self.reliability_ack);
        buffer.put_u16(self.reliability_bits);
    }
}