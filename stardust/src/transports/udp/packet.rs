/// A byte representing what kind of packet is being sent.
#[derive(Debug, Hash, PartialEq, Eq)]
pub(super) enum PacketKind {
    /// Related to managing the UDP connection.
    ConnectionManagement,
    /// Contains a single octet string on one channel.
    /// Apart from the header, the rest of the packet will be considered part of the octet string.
    SingleMessage,
}

impl TryFrom<u8> for PacketKind {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        return Ok(match value {
            0 => Self::ConnectionManagement,
            1 => Self::SingleMessage,
            _ => { return Err(()) }
        })
    }
}


impl Into<u8> for PacketKind {
    fn into(self) -> u8 {
        match self {
            PacketKind::ConnectionManagement => 0,
            PacketKind::SingleMessage => 1,
        }
    }
}

#[test]
fn mgmt_is_zero() {
    // The connection management packet must always be zero, so we have a test for it.
    // If it's not, connection attempts with different transport versions will be ignored.
    assert_eq!(Into::<u8>::into(PacketKind::ConnectionManagement), 0);
    assert_eq!(PacketKind::try_from(0u8).unwrap(), PacketKind::ConnectionManagement);
}