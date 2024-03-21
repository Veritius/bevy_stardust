// A big space for impl blocks.

// TODO: When try_trait_v2 stabilises, this code can be massively simplified using the try operator (?)

use bytes::BufMut;
use unbytes::{EndOfInput, Reader};
use crate::{appdata::NetworkVersionData, connection::handshake::codes::HandshakeResponseCode};
use super::packets::*;

macro_rules! try_read {
    ($value:expr) => {
        match $value {
            Ok(v) => v,
            Err(e) => { return e.into(); }
        }
    };
}

impl HandshakePacketHeader {
    pub fn from_bytes(reader: &mut Reader) -> Result<Self, EndOfInput> {
        return Ok(Self {
            sequence: u16::from_be_bytes(reader.read_array::<2>()?),
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
    fn from_bytes(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        HandshakeParsingResponse::Continue(Self {
            transport: try_read!(NetworkVersionData::from_bytes(reader)),
            application: try_read!(NetworkVersionData::from_bytes(reader)),
        })
    }

    fn write_bytes(&self, buffer: &mut impl BufMut) {
        buffer.put(&self.transport.to_bytes()[..]);
        buffer.put(&self.application.to_bytes()[..]);
    }
}

impl HandshakePacket for ServerHelloPacket {
    fn from_bytes(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        // Check the response code
        let response = u16::from_be_bytes(try_read!(reader.read_array::<2>()));
        let response = HandshakeResponseCode::from(response);
        if response != HandshakeResponseCode::Continue {
            return HandshakeParsingResponse::TheyRejected(response)
        }

        let transport = try_read!(NetworkVersionData::from_bytes(reader));
        let application = try_read!(NetworkVersionData::from_bytes(reader));

        let reliability_ack = u16::from_be_bytes(try_read!(reader.read_array::<2>()));
        let reliability_bits = u16::from_be_bytes(try_read!(reader.read_array::<2>()));

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
    fn from_bytes(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        // Check the response code
        let response = u16::from_be_bytes(try_read!(reader.read_array::<2>()));
        let response = HandshakeResponseCode::from(response);
        if response != HandshakeResponseCode::Continue {
            return HandshakeParsingResponse::TheyRejected(response)
        }

        return HandshakeParsingResponse::Continue(Self {
            reliability_ack: u16::from_be_bytes(try_read!(reader.read_array::<2>())),
            reliability_bits: u16::from_be_bytes(try_read!(reader.read_array::<2>())),
        })
    }

    fn write_bytes(&self, buffer: &mut impl BufMut) {
        // Write response code
        buffer.put_u16(HandshakeResponseCode::Continue as u16);

        buffer.put_u16(self.reliability_ack);
        buffer.put_u16(self.reliability_bits);
    }
}