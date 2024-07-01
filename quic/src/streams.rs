use std::mem::swap as mem_swap;
use bevy::utils::smallvec::SmallVec;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use quinn_proto::{SendStream, VarInt, WriteError, coding::Codec};

pub(crate) trait StreamWrite {
    fn write(&mut self, data: &[u8]) -> StreamWriteOutcome;
}

pub(crate) enum StreamWriteOutcome {
    Complete,
    Partial(usize),
    Error(WriteError),
}

impl StreamWrite for BytesMut {
    fn write(&mut self, data: &[u8]) -> StreamWriteOutcome {
        self.put(data);
        StreamWriteOutcome::Complete
    }
}

impl StreamWrite for SendStream<'_> {
    fn write(&mut self, data: &[u8]) -> StreamWriteOutcome {
        match self.write(&data) {
            Ok(written) if written == data.len() => StreamWriteOutcome::Complete,
            Ok(written) => StreamWriteOutcome::Partial(written),
            Err(err) => StreamWriteOutcome::Error(err),
        }
    }
}

pub(crate) struct StreamWriter {
    queue: SmallVec<[Bytes; 2]>,
}

impl StreamWriter {
    pub fn new() -> Self {
        Self {
            queue: SmallVec::new(),
        }
    }

    #[inline]
    pub fn queue(&mut self, message: Bytes) {
        self.queue.push(message);
    }

    pub fn write<S>(&mut self, stream: &mut S) -> Result<usize, WriteError>
    where
        S: StreamWrite,
    {
        let mut total = 0;
        let mut swap: SmallVec<[Bytes; 2]> = SmallVec::with_capacity(self.queue.len());
        let mut drain = self.queue.drain(..);

        while let Some(bytes) = drain.next() {
            match stream.write(&bytes[..]) {
                // A complete write means we can try again
                StreamWriteOutcome::Complete => {
                    total += bytes.len();
                    continue;
                },

                // A partial write means we have to stop
                StreamWriteOutcome::Partial(written) => {
                    total += written;
                    let bytes = bytes.slice(written..);
                    swap.push(bytes);
                    continue;
                },

                // A block error means we must stop writing
                StreamWriteOutcome::Error(WriteError::Blocked) => {
                    swap.push(bytes);
                    break;
                }

                // An error means the stream can no longer be written to
                StreamWriteOutcome::Error(err) => {
                    swap.push(bytes);
                    return Err(err)
                },
            }
        }

        swap.extend(drain);
        mem_swap(&mut self.queue, &mut swap);
        return Ok(total);
    }
}

pub(crate) enum StreamOpenHeader {
    StardustPersistent {
        channel: u32,
    },

    StardustTransient {
        channel: u32,
    },
}

impl StreamOpenHeader {
    pub fn decode<B: Buf>(buf: &mut B) -> Result<Self, ()> {
        let ident = VarInt::decode(buf).map_err(|_| ())?.into_inner();

        fn decode<B: Buf, T: TryFrom<u64>>(buf: &mut B) -> Result<u32, ()> {
            let varint = VarInt::decode(buf).map_err(|_| ())?.into_inner();
            let value = u32::try_from(varint).map_err(|_| ())?;
            return Ok(value);
        }

        match ident {
            0 => Ok(StreamOpenHeader::StardustPersistent {
                channel: decode::<B, u32>(buf)?,
            }),

            1 => Ok(Self::StardustTransient {
                channel: decode::<B, u32>(buf)?,
            }),

            _ => Err(()),
        }
    }

    pub fn encode<B: BufMut>(&self, buf: &mut B) {
        match self {
            StreamOpenHeader::StardustPersistent { channel } => {
                VarInt::from_u32(0).encode(buf);
                VarInt::from_u32(*channel).encode(buf);
            },

            StreamOpenHeader::StardustTransient { channel } => {
                VarInt::from_u32(1).encode(buf);
                VarInt::from_u32(*channel).encode(buf);
            },
        }
    }
}