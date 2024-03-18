use bytes::Bytes;

/// Management frame types, with an `Ord` implementation comparing how important it is that the frame is sent.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum PacketFrameId {
    Padding = 0,
    Ping = 1,
}

impl TryFrom<u8> for PacketFrameId {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use PacketFrameId::*;
        Ok(match value {
            0 => Padding,
            1 => Ping,
            _ => { return Err(()) }
        })
    }
}

pub(super) struct PacketFrame {
    pub id: PacketFrameId,
    pub pld: Bytes,
}

impl std::fmt::Debug for PacketFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PacketFrame")
        .field("identifier", &self.id)
        .field("pld length", &self.pld.len())
        .finish()
    }
}

impl PartialEq for PacketFrame {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for PacketFrame {}