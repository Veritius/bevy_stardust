use crate::transports::udp::{connections::PendingIncoming, reliability::Reliability};

pub(super) fn process_pending_incoming(
    message: &[u8],
    incoming: &mut PendingIncoming,
    reliability: &mut Reliability,
) {
    use crate::transports::udp::connections::PendingIncomingState;

    match incoming.state {
        PendingIncomingState::JustRegistered => {
            
        },
        PendingIncomingState::Accepted => todo!(),
    }
}