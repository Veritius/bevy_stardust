use std::{future::Future, sync::{Arc, Mutex}};
use async_executor::Executor;
use crate::config::*;

pub(crate) fn create(
    executor: Arc<Executor<'static>>,
    auth: ServerAuthentication,
    verify: ClientVerification,
) -> (EndpointKey, EndpointRef) {
    todo!()
}

#[derive(Clone)]
pub(crate) struct EndpointRef {
    pub(super) inner: Arc<Mutex<EndpointInner>>,
}

pub(crate) struct EndpointKey {
    dropped: Arc<()>,
}

pub(super) struct EndpointInner {
    state: EndpointState,
    dropped: Arc<()>,
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