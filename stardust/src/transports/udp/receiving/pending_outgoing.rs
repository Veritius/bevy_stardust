use crate::transports::udp::{connections::PendingOutgoing, reliability::Reliability};

pub(super) fn process_pending_outgoing(
    message: &[u8],
    outgoing: &mut PendingOutgoing,
    reliability: &mut Reliability,
) {
    todo!()
}