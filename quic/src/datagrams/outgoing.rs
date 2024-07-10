use bytes::{Bytes, BytesMut};
use crate::streams::{OutgoingStreams, StreamManager};
use super::{header::DatagramHeader, DatagramTryWrite};

pub(crate) struct OutgoingDatagrams {

}

impl OutgoingDatagrams {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn send<D: DatagramTryWrite, S: StreamManager>(
        &mut self,
        payload: Bytes,
        dgrams: &mut D,
        strmgr: &mut S,
        streams: &mut OutgoingStreams,
    ) -> anyhow::Result<()> {
        // Create the datagram header
        let header = DatagramHeader {
            purpose: todo!(),
        };

        // Check if the message can be sent in a datagram
        let len = header.encode_len() + payload.len(); 
        match dgrams.datagram_max_size() >= len {
            // The datagram fits and can be sent normally
            true => {
                // Put the header and payload into a single contiguous allocation
                let mut buf = BytesMut::with_capacity(len);
                header.encode(&mut buf).unwrap();
                buf.extend_from_slice(&payload[..]);

                // Try to send the datagram
                dgrams.try_send_datagram(payload)?;
            },

            // The datagram does not fit and must be sent in a stream
            false => todo!(),
        }

        return Ok(());
    }
}