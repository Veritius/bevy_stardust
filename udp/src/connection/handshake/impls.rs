// A big space for impl blocks.

// TODO: When try_trait_v2 stabilises, this code can be massively simplified using the try operator (?)

use bytes::BufMut;
use untrusted::*;
use crate::{appdata::NetworkVersionData, connection::handshake::codes::HandshakeResponseCode, utils::{slice_to_array, IntegerFromByteSlice}};
use super::packets::*;

// Breaks with HandshakeParsingResponse::WeClosed(HandshakeResponseCode::MalformedPacket) if an Err is encountered
// This is easier than repeating a thousand match statements. Remove this when try_trait_v2 stabilises.
macro_rules! try_read {
    ($st:expr) => {
        match $st {
            Ok(val) => val,
            Err(_) => { return HandshakeParsingResponse::WeRejected(HandshakeResponseCode::MalformedPacket) }
        }
    };
}

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
    fn from_reader(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        let transport = NetworkVersionData::from_bytes(try_read!(slice_to_array::<16>(reader)));
        let application = NetworkVersionData::from_bytes(try_read!(slice_to_array::<16>(reader)));

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
    fn from_reader(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        let header = try_read!(HandshakePacketHeader::from_bytes(reader));

        // Check the response code
        let response = try_read!(u16::from_byte_slice(reader)).into();
        if response != HandshakeResponseCode::Continue {
            return HandshakeParsingResponse::TheyRejected(response)
        }

        let transport = NetworkVersionData::from_bytes(try_read!(slice_to_array::<16>(reader)));
        let application = NetworkVersionData::from_bytes(try_read!(slice_to_array::<16>(reader)));

        let reliability_ack = try_read!(u16::from_byte_slice(reader));
        let reliability_bits = try_read!(u16::from_byte_slice(reader));

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
    fn from_reader(reader: &mut Reader) -> HandshakeParsingResponse<Self> {
        let header = try_read!(HandshakePacketHeader::from_bytes(reader));

        // Check the response code
        let response = try_read!(u16::from_byte_slice(reader)).into();
        if response != HandshakeResponseCode::Continue {
            return HandshakeParsingResponse::TheyRejected(response)
        }

        let reliability_ack = try_read!(u16::from_byte_slice(reader));
        let reliability_bits = try_read!(u16::from_byte_slice(reader));

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