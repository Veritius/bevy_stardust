use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct ConnectionRef {
    inner: Arc<Mutex<ConnectionInner>>,
}

pub(super) struct ConnectionInner {
    state: ConnectionState,
}

enum ConnectionState {
    Building(Building),
    Established(Established),
    Shutdown(Shutdown),
}

struct Building {

}

struct Established {
    quinn_state: quinn_proto::Connection,
}

struct Shutdown {

}