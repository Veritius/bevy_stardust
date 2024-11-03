use std::future::Future;
use bevy_stardust::prelude::*;
use crossbeam_channel::{Receiver, Sender};
use quinn_proto::{ConnectionEvent, EndpointEvent};

pub(super) struct Connection {
    quinn_state: quinn_proto::Connection,

    ctrl_events: Receiver<LocalConAppEvent>,
    state_events: Sender<LocalConChgEvent>,

    conn_events: Receiver<ConnectionEvent>,
    endp_events: Sender<EndpointEvent>,

    inc_messages: Receiver<ChannelMessage>,
    out_messages: Sender<ChannelMessage>,
}

impl Future for Connection {
    type Output = Result<(), ConnectionError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

pub(crate) struct ConnectionHandle {
    ctrl_events: Sender<LocalConAppEvent>,
    state_events: Receiver<LocalConChgEvent>,

    inc_messages: Sender<ChannelMessage>,
    out_messages: Receiver<ChannelMessage>,
}

pub(crate) enum LocalConAppEvent {

}

pub(crate) enum LocalConChgEvent {

}

/// A connection that is being created.
pub(crate) struct ConnectionCreation {

}

impl Future for ConnectionCreation {
    type Output = Result<ConnectionHandle, ConnectionError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

/// Error returned when creating a connection.
pub(crate) enum ConnectionError {

}