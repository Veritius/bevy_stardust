use crate::{channel::Channel, types::NetworkUserId, bits::{ManualBitSerialisation, BitReader, BitWriter, BitstreamError}};

/// Special channel that is used by the client/server when first joining.
#[derive(Debug)]
pub(crate) struct AuthenticationChannel;
impl Channel for AuthenticationChannel {}

enum AuthenticationMessage {
    /// Sent from the client to the server, requesting acknowledgement.
    ConnectRequest,
    /// Sent from the server to the client, sending a new ID.
    ConnectResponse(NetworkUserId),
}

impl ManualBitSerialisation for AuthenticationMessage {
    fn serialise(&self, writer: &mut impl BitWriter) {
        match self {
            AuthenticationMessage::ConnectRequest => {
                writer.allocate_bytes(1);
                writer.write_byte(0);
                writer.write_bit(false);
                writer.write_bit(false);
            },
            AuthenticationMessage::ConnectResponse(val) => {
                writer.allocate_bytes(5);
                writer.write_byte(1);
                val.serialise(writer);
            },
        }
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        match reader.read_byte()? {
            0 => Ok(AuthenticationMessage::ConnectRequest),
            1 => {
                let val = u32::deserialise(reader)?;
                Ok(AuthenticationMessage::ConnectResponse(NetworkUserId(val)))
            }
            _ => Err(BitstreamError)
        }
    }
}