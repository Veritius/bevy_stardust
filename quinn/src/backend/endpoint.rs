use std::{future::Future, sync::{Arc, Mutex}};

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

struct EndpointDriver(EndpointRef);

impl Future for EndpointDriver {
    type Output = Result<(), ()>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}