use crate::connection::datagrams::DatagramTryWrite;
use super::QuicheConnection;

impl DatagramTryWrite for QuicheConnection {
    fn datagram_max_size(&self) -> usize {
        self.inner.max_send_udp_payload_size()
    }

    fn try_send_datagram(&mut self, payload: bytes::Bytes) -> anyhow::Result<usize> {
        self.inner.dgram_send(&payload[..])?;
        return Ok(payload.len());
    }
}