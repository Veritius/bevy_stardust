use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}};
use super::endpoint::EndpointInner;

#[derive(Clone)]
pub(crate) struct ConnectionRef {
    ptr: Arc<Mutex<ConnectionInner>>,
}

pub(super) struct ConnectionInner {
    state: ConnectionState,

    waker: Option<Waker>,
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

struct ConnectionDriver(ConnectionRef);

impl Future for ConnectionDriver {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let mut connection = self.0.ptr.lock().unwrap();

        if connection.waker.is_none() {
            connection.waker = Some(cx.waker().clone());
        }

        todo!()
    }
}