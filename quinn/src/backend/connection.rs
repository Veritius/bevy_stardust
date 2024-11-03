use std::{future::Future, sync::{Arc, Mutex}};
use async_executor::Executor;
use crate::config::*;
use super::endpoint::EndpointRef;

pub(crate) fn create(
    executor: Arc<Executor<'static>>,
    endpoint: EndpointRef,
    auth: ClientAuthentication,
    verify: ServerVerification,
    server_name: Arc<str>,
) -> (ConnectionKey, ConnectionRef) {
    todo!()
}

#[derive(Clone)]
pub(crate) struct ConnectionRef {
    pub(super) inner: Arc<Mutex<ConnectionInner>>,
}

pub(crate) struct ConnectionKey {
    dropped: Arc<()>,
}

pub(super) struct ConnectionInner {
    state: ConnectionState,
    dropped: Arc<()>,
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

struct ConnectionDriver(ConnectionRef);

impl Future for ConnectionDriver {
    type Output = Result<(), ()>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}