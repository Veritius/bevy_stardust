pub(super) enum HandshakePacketType {
    ConnectionRequest,
    ConnectionDenial,
    ConnectionAccepted,
}

impl From<HandshakePacketType> for u16 {
    fn from(value: HandshakePacketType) -> Self {
        use HandshakePacketType::*;
        match value {
            ConnectionRequest => 0,
            ConnectionDenial => 1,
            ConnectionAccepted => 2,
        }
    }
}

impl TryFrom<u16> for HandshakePacketType {
    type Error = u16;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        use HandshakePacketType::*;
        Ok(match value {
            0 => ConnectionRequest,
            1 => ConnectionDenial,
            2 => ConnectionAccepted,
            _ => { return Err(value) }
        })
    }
}