pub(crate) struct EndpointInner {
    state: EndpointState,
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