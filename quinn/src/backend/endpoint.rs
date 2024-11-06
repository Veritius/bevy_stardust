use std::{future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}};
use crossbeam_channel::Receiver;
use super::socket::Receive;

#[derive(Clone)]
pub(crate) struct EndpointRef {
    ptr: Arc<Mutex<EndpointInner>>,
}

pub(super) struct EndpointInner {
    state: EndpointState,

    dgrams: Receiver<Receive>,

    waker: Option<Waker>,
}

impl EndpointInner {
    pub fn new(
        config: EndpointConfig,
    ) -> Self {
        todo!()
    }
}

pub(crate) struct EndpointConfig {

}

enum EndpointState {
    Established(Established),
    Shutdown(Shutdown),
}

struct Established {
    quinn_proto: quinn_proto::Endpoint,
}

struct Shutdown {

}

struct EndpointDriver(EndpointRef);

impl Future for EndpointDriver {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let mut endpoint = self.0.ptr.lock().unwrap();

        if endpoint.waker.is_none() {
            endpoint.waker = Some(cx.waker().clone());
        }

        todo!()
    }
}