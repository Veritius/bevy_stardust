use super::endpoint::EndpointInner;

pub(crate) struct ConnectionInner {
    state: ConnectionState,
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

}

struct Established {

}

struct Shutdown {

}