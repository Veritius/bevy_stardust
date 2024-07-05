use bevy_stardust_extras::numbers::{Sequence, VarInt};
use bytes::{Buf, BufMut, Bytes};
use smallvec::SmallVec;

type Seq = Sequence<u32>;

pub(crate) struct DatagramDesequencer {
    waiting: SmallVec<[(Seq, Bytes); 2]>,
}

impl DatagramDesequencer {
    pub fn new() -> Self {
        Self {
            waiting: SmallVec::default(),
        }
    }

    pub fn new_boxed(&self) -> Box<Self> {
        Box::new(Self::new())
    }

    pub fn store(&mut self, seq: Seq, data: Bytes) {
        todo!()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Bytes> {
        [].into_iter() // TODO
    }
}

pub(crate) struct DatagramSequencer {
    send_idx: Seq,
}

impl DatagramSequencer {
    pub fn new() -> Self {
        Self {
            send_idx: Seq::default(),
        }
    }

    pub fn next(&mut self) -> Seq {
        let f = self.send_idx;
        self.send_idx.increment();
        return f;
    }
}

pub(crate) struct Datagram {
    pub header: DatagramHeader,
    pub payload: Bytes,
}

impl Datagram {
    pub fn size(&self) -> usize {
        self.header.size() + self.payload.len()
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        self.header.encode(buf)?;
        if buf.remaining_mut() < self.payload.len() { return Err(()); }
        buf.put(self.payload.clone());
        return Ok(());
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
    pub fn size(&self) -> usize {
        self.purpose.size()
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        self.purpose.encode(buf)
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

    StardustSequenced {
        channel: u32,
        sequence: Seq,
    },
}

impl DatagramPurpose {
    pub fn size(&self) -> usize {
        let mut size: usize = 0;

        match self {
            DatagramPurpose::StardustUnordered { channel } => {
                size += VarInt::from_u32(0).len() as usize;
                size += VarInt::from_u32(*channel).len() as usize;
            },

            DatagramPurpose::StardustSequenced { channel, sequence } => {
                size += VarInt::from_u32(1).len() as usize;
                size += VarInt::from_u32(*channel).len() as usize;
                size += VarInt::from_u32(sequence.inner()).len() as usize;
            },
        }

        return size;
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) -> Result<(), ()> {
        match self {
            DatagramPurpose::StardustUnordered { channel } => {
                VarInt::from_u32(0).write(buf)?;
                VarInt::from_u32(*channel).write(buf)?;
            },

            DatagramPurpose::StardustSequenced { channel, sequence } => {
                VarInt::from_u32(1).write(buf)?;
                VarInt::from_u32(*channel).write(buf)?;
                VarInt::from_u32(sequence.inner()).write(buf)?;
            },
        }

        return Ok(())
    }

    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, ()> {
        let code = VarInt::read(buf)?.into();

        return Ok(match code {
            0 => Self::StardustUnordered {
                channel: decode_uint::<B, u32>(buf)?,
            },

            1 => Self::StardustSequenced {
                channel: decode_uint::<B, u32>(buf)?,
                sequence: decode_uint::<B, u32>(buf)?.into()
            },

            _ => return Err(())
        });
    }
}

fn decode_uint<B: Buf, T: TryFrom<u64>>(buf: &mut B) -> Result<T, ()> {
    let u64: u64 = VarInt::read(buf)?.into();
    let t = <T as TryFrom<u64>>::try_from(u64).map_err(|_| ())?;
    return Ok(t);
}