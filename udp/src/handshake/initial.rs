use untrusted::*;
use super::failure::HandshakeFailureMessage;

pub(crate) fn send_outgoing_initial() -> std::io::Result<()> {
    todo!()
}

/// Process the very first packet received from an unknown peer.
/// 
/// If `match_flag` contains duplicate elements or is above 128 elements in length, this function will always reject the peer.
pub(crate) fn read_incoming_initial(
    bytes: &[u8],
    match_flags: &[&[u8]],
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

    // Read through the handshake flags
    let flag_count = reader.read_byte()? as usize;
    let mut matched_flags = 0u128;
    for idx in 0..flag_count {
        let flag_length = reader.read_byte()? as usize;
        let flag_value = reader.read_bytes(flag_length)?.as_slice_less_safe();

        // Check if the flag is in match_flags
        if let Some(idx) = match_flags.iter().position(|v| *v == flag_value) {
            if idx > 128 { break } // too many match flags
            matched_flags |= 1u128 << idx; // set bit to true
        }
    }

    // Check that we've matched enough handshake flags
    // this is inefficient as written but the compiler probably optimises it so eh
    for bit in 0..128 {
        if 1u128 >> bit & matched_flags > 0 {
            return Ok(InitialPacketOutcome::Rejected(
                HandshakeFailureMessage::MandatoryFlagMismatch,
            ))
        }
    }

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