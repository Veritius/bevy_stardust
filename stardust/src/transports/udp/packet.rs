/// The header of a packet.
pub(super) struct PacketHeader {
    pub kind: PacketKind
}

/// A byte representing what kind of packet is being sent.
#[derive(Debug, Hash, PartialEq, Eq)]
pub(super) enum PacketKind {
    /// Related to managing the UDP connection.
    ConnectionManagement = 0,
    /// Contains a single octet string on one channel.
    SingleMessage = 1,
}