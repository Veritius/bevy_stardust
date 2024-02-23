use quinn_proto::{SendStream, StreamId, VarInt, WriteError};

#[repr(u32)]
pub(crate) enum StreamErrorCode {
    Disconnecting = 0,
}

impl From<StreamErrorCode> for VarInt {
    fn from(value: StreamErrorCode) -> Self {
        Self::from_u32(value as u32)
    }
}

impl From<StreamErrorCode> for Option<VarInt> {
    fn from(value: StreamErrorCode) -> Self {
        Some(value.into())
    }
}

impl TryFrom<VarInt> for StreamErrorCode {
    type Error = VarInt;

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        Ok(match u64::from(value) {
            0 => Self::Disconnecting,
            _ => { return Err(value) }
        })
    }
}

#[repr(u8)]
pub(crate) enum StreamPurposeHeader {
    ConnectionManagement = 0,
    StardustPayloads = 1,
}

impl TryFrom<u8> for StreamPurposeHeader {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::ConnectionManagement,
            1 => Self::StardustPayloads,
            _ => return Err(())
        })
    }
}

pub(crate) struct OutgoingBufferedStreamData {
    pub id: StreamId,
    buffer: Box<[u8]>,
}

impl OutgoingBufferedStreamData {
    pub fn new(id: StreamId) -> Self {
        Self {
            id,
            buffer: Box::default(),
        }
    }

    pub fn push(&mut self, data: &[u8]) {
        if data.len() == 0 { return }

        let mut buf: Vec<u8> = Vec::with_capacity(self.buffer.len() + data.len());
        buf.extend_from_slice(&self.buffer);
        buf.extend_from_slice(data);

        self.buffer = buf.into();
    }

    pub fn try_write(&mut self, stream: &mut SendStream) -> Result<usize, WriteError> {
        match stream.write(&self.buffer) {
            Ok(bytes) => {
                if bytes == 0 { return Ok(bytes) }
                let mut buf = Vec::with_capacity(self.buffer.len() - bytes);
                buf.extend(&self.buffer[bytes..]);
                self.buffer = buf.into();
                Ok(bytes)
            },
            Err(err) => Err(err),
        }
    }

    pub fn is_drained(&self) -> bool {
        self.buffer.len() == 0
    }
}