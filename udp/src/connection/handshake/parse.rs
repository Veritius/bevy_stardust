use unbytes::*;
use crate::sequences::SequenceId;
use super::Handshaking;

pub(super) fn parse_header(
    this: &mut Handshaking,
    reader: &mut Reader,
) -> Result<(), ParseError> {
    // Read the packet sequence identifier
    let seq: SequenceId = reader.read_u16()?.into();

    // If the packet is too old ignore it
    if seq <= this.shared.reliability.remote_sequence {
        return Err(ParseError::Outdated);
    }

    return Ok(());
}

pub(super) enum ParseError {
    EndOfInput(EndOfInput),
    Outdated,
}

impl From<EndOfInput> for ParseError {
    fn from(value: EndOfInput) -> Self {
        Self::EndOfInput(value)
    }
}