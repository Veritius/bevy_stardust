use bytes::Bytes;
use crate::datagrams::header::{DatagramHeader, DatagramPurpose};
use super::{Datagram, DatagramTag};

pub(crate) struct IncomingDatagrams {

}

impl IncomingDatagrams {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn recv(&mut self, mut bytes: Bytes) -> anyhow::Result<Datagram> {
        // Decode the datagram header
        let header = DatagramHeader::decode(&mut bytes).map_err(|_| anyhow::anyhow!("Failed to decode datagram header"))?;

        return Ok(Datagram {
            // Match the tag from the purpose header
            tag: match header.purpose {
                DatagramPurpose::Stardust { channel } => DatagramTag::Stardust { channel, sequence: None },
                DatagramPurpose::StardustSequenced { channel, sequence } => DatagramTag::Stardust { channel, sequence: Some(sequence.into()) },
            },

            // The rest of the datagram is the payload
            payload: bytes
        });
    }
}