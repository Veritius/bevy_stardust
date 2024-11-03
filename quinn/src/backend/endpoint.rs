use crossbeam_channel::{Receiver, Sender};
use quinn_proto::{ConnectionEvent, EndpointEvent};

pub(super) struct Endpoint {
    quinn_state: quinn_proto::Endpoint,

    conn_events: Sender<ConnectionEvent>,
    endp_events: Receiver<EndpointEvent>,
}