use bytes::Bytes;

pub(crate) struct IncomingPacket {
    pub payload: Bytes,
}

pub(crate) struct OutgoingPacket {
    pub payload: Bytes,
    pub messages: u32,
}