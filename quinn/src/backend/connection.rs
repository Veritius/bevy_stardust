use std::{future::Future, pin::Pin, sync::{mpsc::Sender, Arc, Mutex}, task::{Context, Poll, Waker}};
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
    endpoint: EndpointRef,

    waker: Option<Waker>,
}

struct EndpointHandle {
    endpoint_ref: EndpointRef,
    recv_events: Receiver<quinn_proto::ConnectionEvent>,
    send_events: Sender<quinn_proto::EndpointEvent>,
}

struct ConnectionDriver(ConnectionRef);

impl Future for ConnectionDriver {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let mut connection = self.0.ptr.lock().unwrap();

        if connection.shared.waker.is_none() {
            connection.shared.waker = Some(cx.waker().clone());
        }

        todo!()
    }
}