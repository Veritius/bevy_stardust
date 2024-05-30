use unbytes::*;
use crate::sequences::SequenceId;
use super::Handshaking;

fn parse_header(
    this: &mut Handshaking,
    mut reader: Reader,
) -> Result<Reader, ParseError> {
    // Read the packet sequence identifier
    let seq: SequenceId = reader.read_u16()?.into();

    // If the packet is too old ignore it
    if seq <= this.shared.reliability.remote_sequence {
        return Err(ParseError::Outdated);
    }

    return Ok(reader);
}

enum ParseError {
    EndOfInput(EndOfInput),
    Outdated,
}

impl From<EndOfInput> for ParseError {
    fn from(value: EndOfInput) -> Self {
        Self::EndOfInput(value)
    }
}