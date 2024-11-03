use std::future::Future;
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

/// An endpoint that is being created.
pub(crate) struct EndpointCreation {

}

impl Future for EndpointCreation {
    type Output = Result<EndpointHandle, EndpointError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

/// Error returned when creating an endpoint.
pub(crate) enum EndpointError {

}