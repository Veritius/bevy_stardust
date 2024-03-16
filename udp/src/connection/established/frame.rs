use bytes::Bytes;

/// Frame types, with an `Ord` implementation comparing how important it is that the frame is sent.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum PacketFrameId {
    Padding = 0,
    Ping = 1,
    Payload = 2,
    Management = 3,
    Acknowledgement = 4,
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

impl PartialOrd for PacketFrame {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for PacketFrame {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}