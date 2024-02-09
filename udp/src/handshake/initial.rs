use untrusted::*;
use super::failure::HandshakeFailureMessage;

const ENCRYPTION_REQUIRED: u16 = 0b0000_0000_0001;
const ENCRYPTION_USE_X509: u16 = 0b0000_0000_0010;

pub(crate) fn send_outgoing_initial() -> std::io::Result<()> {
    todo!()
}

/// Process the very first packet received from an unknown peer.
pub(crate) fn read_incoming_initial(
    bytes: &[u8],
) -> Result<InitialPacketOutcome, EndOfInput> {
    let mut reader = Reader::new(Input::from(bytes));

    // Read their version value to see if we support their client
    let version_length = reader.read_byte()? as usize;
    let version_value = reader.read_bytes(version_length)?;
    if !super::ACCEPTABLE_TRANSPORT_VERSIONS.contains(&version_value.as_slice_less_safe()) {
        // return Ok(InitialPacketOutcome::Rejected {})
        todo!();
    }

    // Read their starting sequence id for reliability
    // Safety: Since Reader already checks that the slice has a length of 2, the TryInto call shouldn't fail
    let seq_bytes = TryInto::<[u8;2]>::try_into(reader.read_bytes(2)?.as_slice_less_safe()).unwrap();
    let seq = u16::from_be_bytes(seq_bytes);

    // Check any encryption related values
    let encryption_flags = u16::from_be_bytes(TryInto::<[u8;2]>::try_into(reader.read_bytes(2)?.as_slice_less_safe()).unwrap());
    let encryption_required = (encryption_flags | ENCRYPTION_REQUIRED) > 0;

    #[cfg(not(feature="encryption"))]
    if encryption_required {
        // We don't support encryption, so we reject them immediately.
        return Ok(HandshakeFailureMessage::EncryptionNotSupported.into());
    }

    // #[cfg(feature="encryption")]
    // if encryption_required && true {
    //     todo!();
    // }

    // They've passed all the checks for this stage of the transaction
    return Ok(InitialPacketOutcome::Continue {
        seq,
    })
}

pub(crate) enum InitialPacketOutcome {
    /// Continue to further into the handshake.
    Continue {
        seq: u16,
    },

    /// Inform them that we don't want to connect.
    Rejected(HandshakeFailureMessage),
    
}

impl From<HandshakeFailureMessage> for InitialPacketOutcome {
    fn from(value: HandshakeFailureMessage) -> Self {
        Self::Rejected(value)
    }
}