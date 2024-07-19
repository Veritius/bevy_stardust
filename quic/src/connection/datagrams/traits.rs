use bytes::Bytes;

pub(crate) trait DatagramTryWrite {
    fn datagram_max_size(&self) -> usize;
    fn try_send_datagram(&mut self, payload: Bytes) -> anyhow::Result<usize>;
}