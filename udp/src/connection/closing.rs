use bytes::Bytes;

pub(super) struct Closing {
    pub reason: Option<Bytes>,
}