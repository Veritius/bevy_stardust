use crate::connection::reliability::ReliablePackets;

pub(super) struct River {
    reliability: ReliablePackets,
}