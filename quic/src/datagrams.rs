use bytes::{Buf, BufMut};
use quinn_proto::{VarInt, coding::Codec};

pub(crate) struct DatagramHeader {
    pub purpose: DatagramPurpose,
    pub length: u32,
}

impl DatagramHeader {
    pub fn decode<B: Buf>(b: &mut B) -> Result<Self, DatagramHeaderParseError> {
        let purpose = DatagramPurpose::decode(b)?;
        let length: u32 = decode_varint(b)?.try_into().map_err(|_| DatagramHeaderParseError::ExceededMaxLength)?;
        return Ok(Self { purpose, length })
    }

    pub fn encode<B: BufMut>(&self, b: &mut B) {
        self.purpose.encode(b);
        VarInt::from_u32(self.length).encode(b);
    }
}

pub(crate) enum DatagramPurpose {
    Stardust,
}

impl DatagramPurpose {
    pub fn decode<B: Buf>(b: &mut B) -> Result<Self, DatagramHeaderParseError> {
        let code = decode_varint(b)?;

        match code {
            0 => return Ok(Self::Stardust),
            _ => return Err(DatagramHeaderParseError::InvalidPurposeCode),
        }
    }

    pub fn encode<B: BufMut>(&self, b: &mut B) {
        VarInt::from_u32(match self {
            DatagramPurpose::Stardust => 0,
        }).encode(b);
    }
}

pub(crate) enum DatagramHeaderParseError {
    EndOfInput,
    InvalidPurposeCode,
    ExceededMaxLength,
}

#[inline]
fn decode_varint<B: Buf>(b: &mut B) -> Result<u64, DatagramHeaderParseError> {
    VarInt::decode(b)
        .map(|v| v.into_inner())
        .map_err(|_| DatagramHeaderParseError::EndOfInput)
}