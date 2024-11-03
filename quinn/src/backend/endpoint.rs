use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct EndpointRef {
    inner: Arc<Mutex<EndpointInner>>,
}

pub(super) struct EndpointInner {
    state: EndpointState,
}

enum EndpointState {
    Building(Building),
    Established(Established),
    Shutdown(Shutdown),
}

struct Building {

}

struct Established {
    quinn_state: quinn_proto::Endpoint,
}

struct Shutdown {

}