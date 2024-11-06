use std::sync::{Arc, Mutex};
use crossbeam_channel::Receiver;
use super::socket::Receive;

#[derive(Clone)]
pub(crate) struct EndpointRef {
    ptr: Arc<Mutex<EndpointInner>>,
}

pub(super) struct EndpointInner {
    state: EndpointState,

    dgrams: Receiver<Receive>,
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