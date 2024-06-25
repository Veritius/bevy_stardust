use bytes::{Buf, BufMut, Bytes};
use quinn_proto::{VarInt, coding::Codec};

pub(crate) struct DatagramQueue {
    queue: Vec<PendingDatagram>,
}

impl DatagramQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, datagram: PendingDatagram) {
        self.queue.push(datagram);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = PendingDatagram> + '_ {
        self.queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        struct DatagramQueueDrain<'a> {
            inner: &'a mut DatagramQueue,
        }

        impl Iterator for DatagramQueueDrain<'_> {
            type Item = PendingDatagram;

            fn next(&mut self) -> Option<Self::Item> {
                self.inner.queue.pop()
            }
        }

        return DatagramQueueDrain { inner: self }
    }
}

pub(crate) struct PendingDatagram {
    pub priority: u32,
    pub purpose: DatagramPurpose,
    pub payload: Bytes,
}

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

#[test]
fn datagram_queue_sorting_test() {
    let mut queue = DatagramQueue::with_capacity(3);
    queue.push(PendingDatagram { priority: 0, purpose: DatagramPurpose::Stardust, payload: Bytes::new() });
    queue.push(PendingDatagram { priority: 1, purpose: DatagramPurpose::Stardust, payload: Bytes::new() });
    queue.push(PendingDatagram { priority: 2, purpose: DatagramPurpose::Stardust, payload: Bytes::new() });

    let mut last_priority: u32 = 0;
    for item in queue.drain() {
        assert!(item.priority >= last_priority);
        last_priority = item.priority;
    }
}