use bytes::{Buf, BufMut, Bytes};
use quinn_proto::{VarInt, coding::Codec};

pub(crate) struct Datagram {
    pub header: DatagramHeader,
    pub payload: Bytes,
}

impl Datagram {
    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        self.header.encode(buf);
        buf.put(self.payload.clone());
    }

    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, ()> {
        return Ok(Self {
            header: DatagramHeader::decode(buf)?,
            payload: buf.copy_to_bytes(buf.remaining()),
        });
    }
}

pub(crate) struct DatagramHeader {
    pub purpose: DatagramPurpose,
}

impl DatagramHeader {
    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        self.purpose.encode(buf);
    }

    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, ()> {
        return Ok(Self {
            purpose: DatagramPurpose::decode(buf)?,
        });
    }
}

pub(crate) enum DatagramPurpose {
    StardustUnordered {
        channel: u32,
    },

    StardustOrdered {
        channel: u32,
        order: u32,
    },
}

impl DatagramPurpose {
    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        match self {
            DatagramPurpose::StardustUnordered { channel } => {
                VarInt::from_u32(0).encode(buf);
                VarInt::from_u32(*channel).encode(buf);
            },

            DatagramPurpose::StardustOrdered { channel, order } => {
                VarInt::from_u32(0).encode(buf);
                VarInt::from_u32(*channel).encode(buf);
                VarInt::from_u32(*order).encode(buf);
            },
        }
    }

    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, ()> {
        let code = decode_varint(buf)?.into_inner();

        return Ok(match code {
            0 => Self::StardustUnordered {
                channel: decode_uint::<B, u32>(buf)?,
            },

            1 => Self::StardustOrdered {
                channel: decode_uint::<B, u32>(buf)?,
                order: decode_uint::<B, u32>(buf)?
            },

            _ => return Err(())
        });
    }
}

fn decode_varint<B: Buf>(buf: &mut B) -> Result<VarInt, ()> {
    VarInt::decode(buf).map_err(|_| ())
}

fn decode_uint<B: Buf, T: TryFrom<u64>>(buf: &mut B) -> Result<T, ()> {
    let u64 = decode_varint(buf)?.into_inner();
    let t = <T as TryFrom<u64>>::try_from(u64).map_err(|_| ())?;
    return Ok(t);
}