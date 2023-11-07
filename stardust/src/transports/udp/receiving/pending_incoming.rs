use crate::transports::udp::{connections::{PendingIncoming, PendingIncomingState, Disconnected}, reliability::Reliability};

pub(super) fn process_pending_incoming(
    message: &[u8],
    incoming: &mut PendingIncoming,
    reliability: &mut Reliability,
) {
    incoming.state = match incoming.state {
        PendingIncomingState::JustRegistered => read_initial_packet(message),
        PendingIncomingState::Accepted => todo!(),
        PendingIncomingState::Rejected(_) => todo!(),
    }
}

fn read_initial_packet(
    message: &[u8],
) -> PendingIncomingState {
    // Try to parse into a string slice
    let string = match std::str::from_utf8(message) {
        Ok(v) => v,
        Err(_) => {
            return PendingIncomingState::Rejected(Disconnected::InvalidPacket)
        },
    };

    // Try to parse into a json document
    let json = match json::parse(string) {
        Ok(v) => v,
        Err(_) => {
            return PendingIncomingState::Rejected(Disconnected::InvalidPacket);
        },
    };

    // They've succeeded :)
    return PendingIncomingState::Accepted
}