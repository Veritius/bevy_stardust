use crate::transports::udp::{connections::{PendingIncoming, PendingIncomingState, Disconnected}, reliability::Reliability, COMPAT_GOOD_VERSIONS, ordering::OrderingData, TRANSPORT_IDENTIFIER};

pub(super) fn process_pending_incoming(
    message: &[u8],
    incoming: &mut PendingIncoming,
    reliability: &mut Reliability,
    ordering: &mut OrderingData,
    protocol: u64,
) {
    incoming.state = match incoming.state {
        PendingIncomingState::JustRegistered => read_initial_packet(message, protocol),
        PendingIncomingState::Accepted => todo!(),
        PendingIncomingState::Rejected(_) => todo!(),
    }
}

fn read_initial_packet(
    message: &[u8],
    protocol: u64,
) -> PendingIncomingState {
    // Basic requirements for an initial packet
    if message.len() < 20 {
        return PendingIncomingState::Rejected(Disconnected::HandshakeMalformedPacket) }

    let identifier = u64::from_be_bytes(message[0..8].try_into().unwrap());
    if identifier != TRANSPORT_IDENTIFIER {
        return PendingIncomingState::Rejected(Disconnected::HandshakeUnknownTransport { identifier }) }

    let version = u32::from_be_bytes(message[8..12].try_into().unwrap());
    if !COMPAT_GOOD_VERSIONS.contains(&version) {
        return PendingIncomingState::Rejected(Disconnected::HandshakeWrongVersion { version }) }

    let other_protocol = u64::from_be_bytes(message[12..20].try_into().unwrap());
    if other_protocol != protocol {
        return PendingIncomingState::Rejected(Disconnected::HandshakeWrongProtocol { protocol }) }

    // They've succeeded :)
    return PendingIncomingState::Accepted
}