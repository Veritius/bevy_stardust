use crate::transports::udp::{connections::{PendingIncoming, PendingIncomingState, Disconnected}, reliability::Reliability, COMPAT_GOOD_VERSIONS, ordering::OrderingData, TRANSPORT_IDENTIFIER};

pub(super) fn process_pending_incoming(
    message: &[u8],
    incoming: &mut PendingIncoming,
    reliability: &mut Reliability,
    ordering: &mut OrderingData,
    protocol: u64,
) {
    incoming.state = match incoming.state {
        PendingIncomingState::JustRegistered => read_initial_packet(message, protocol, reliability),
        PendingIncomingState::Accepted => todo!(),
        PendingIncomingState::Rejected(_) => todo!(),
    }
}

fn read_initial_packet(
    message: &[u8],
    protocol: u64,
    reliability: &mut Reliability,
) -> PendingIncomingState {
    // Basic requirements for an initial packet
    if message.len() < 22 {
        return Disconnected::HandshakeMalformedPacket.into()
    }

    let identifier = u64::from_be_bytes(message[0..8].try_into().unwrap());
    if identifier != TRANSPORT_IDENTIFIER {
        return Disconnected::HandshakeUnknownTransport { identifier }.into() }

    let version = u32::from_be_bytes(message[8..12].try_into().unwrap());
    if !COMPAT_GOOD_VERSIONS.contains(&version) {
        return Disconnected::HandshakeWrongVersion { version }.into() }

    let other_protocol = u64::from_be_bytes(message[12..20].try_into().unwrap());
    if other_protocol != protocol {
        return Disconnected::HandshakeWrongProtocol { protocol }.into() }

    // Use their sequence id in Reliability's remote field
    reliability.remote = u16::from_be_bytes(message[20..22].try_into().unwrap());

    // They've succeeded :)
    return PendingIncomingState::Accepted
}