use bytes::{Bytes, BytesMut};
use crate::streams::{OutgoingStreams, OutgoingStreamsTryWriteOutcome, StreamManager, StreamPurpose, StreamTryWriteOutcome};
use super::{header::{DatagramHeader, DatagramPurpose}, DatagramTryWrite};

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
        purpose: DatagramPurpose,
        dgrams: &mut D,
        strmgr: &mut S,
        streams: &mut OutgoingStreams,
    ) -> anyhow::Result<()> {
        // Create the datagram header
        let header = DatagramHeader {
            purpose,
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
            false => {
                // Open a new transient stream to wrap our datagram
                let purpose = StreamPurpose::Datagram;
                let id = strmgr.open_send_stream()?;
                let mut outgoing = streams.open_and_get(id, purpose, true);

                // Encode the header into its own allocation
                let mut buf = BytesMut::with_capacity(len);
                header.encode(&mut buf).unwrap();
                let header = buf.freeze();

                // Push the header and the payload into the queue for sending
                outgoing.push_chunks_framed([header, payload].iter().cloned());

                // Try to send as much as possible on the stream
                let mut transmit = strmgr.get_send_stream(id).unwrap();
                match streams.write(id, &mut transmit) {
                    // The stream was finished
                    Some(OutgoingStreamsTryWriteOutcome::Finished(_)) => todo!(),

                    // An error occurred
                    Some(OutgoingStreamsTryWriteOutcome::WriteOutcome(StreamTryWriteOutcome::Error(err))) => todo!(),

                    _ => { /* Do nothing */ },
                }
            },
        }

        return Ok(());
    }
}