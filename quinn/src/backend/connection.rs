use std::{future::Future, pin::Pin, sync::{mpsc::Sender, Arc, Mutex, Weak}, task::{Context, Poll, Waker}};
use crossbeam_channel::Receiver;
use super::endpoint::{EndpointInner, EndpointRef};

#[derive(Clone)]
pub(crate) struct ConnectionRef {
    ptr: Arc<Mutex<ConnectionInner>>,
}

pub(super) struct ConnectionInner {
    state: ConnectionState,
    shared: Shared,
}

impl ConnectionInner {
    pub fn new(
        endpoint: &mut EndpointInner,
        config: ConnectionConfig,
    ) -> Self {
        todo!()
    }
}

pub(crate) struct ConnectionConfig {

}

enum ConnectionState {
    Connecting(Connecting),
    Established(Established),
    Shutdown(Shutdown),
}

struct Connecting {
    quinn_state: quinn_proto::Connection,
}

struct Established {
    quinn_state: quinn_proto::Connection,
}

struct Shutdown {

}

struct Shared {
    endpoint: Weak<Mutex<EndpointInner>>,

    waker: Option<Waker>,
}

struct EndpointHandle {
    waker: Waker,

    recv_events: Receiver<quinn_proto::ConnectionEvent>,
    send_events: Sender<quinn_proto::EndpointEvent>,
}