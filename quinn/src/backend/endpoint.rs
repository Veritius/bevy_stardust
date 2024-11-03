use crossbeam_channel::{Receiver, Sender};
use quinn_proto::{ConnectionEvent, EndpointEvent};

pub(super) struct Endpoint {
    quinn_state: quinn_proto::Endpoint,

    ctrl_events: Receiver<LocalEndAppEvent>,
    state_events: Sender<LocalEndChgEvent>,

    conn_events: Sender<ConnectionEvent>,
    endp_events: Receiver<EndpointEvent>,
}

pub(crate) struct EndpointHandle {
    ctrl_events: Sender<LocalEndAppEvent>,
    state_events: Receiver<LocalEndChgEvent>,
}

pub(crate) enum LocalEndAppEvent {

}

pub(crate) enum LocalEndChgEvent {

}