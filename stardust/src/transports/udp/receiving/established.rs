use crate::transports::udp::{connections::Established, reliability::Reliability, ordering::Ordering};

pub(super) fn process_established(
    message: &[u8],
    established: &mut Established,
    reliability: &mut Reliability,
    ordering: &mut Ordering,
) {
    todo!()
}