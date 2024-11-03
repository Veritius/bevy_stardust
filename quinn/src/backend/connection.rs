use std::{future::Future, pin::{Pin, pin}, sync::{Arc, Mutex}, task::Poll};
use async_executor::Executor;
use crate::config::*;
use super::endpoint::EndpointRef;

pub(crate) fn create(
    executor: Arc<Executor<'static>>,
    endpoint: EndpointRef,
    auth: ClientAuthentication,
    verify: ServerVerification,
    server_name: Arc<str>,
) -> ConnectionRef {
    todo!()
}

#[derive(Clone)]
pub(crate) struct ConnectionRef {
    pub(super) inner: Arc<Mutex<ConnectionInner>>,
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

impl Future for Building {
    type Output = Result<Established, Shutdown>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

struct Established {
    quinn_state: quinn_proto::Connection,
}

impl Future for Established {
    type Output = Shutdown;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

struct Shutdown {

}

struct ConnectionDriver(ConnectionRef);

impl Future for ConnectionDriver {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut inner = self.0.inner.lock().unwrap();

        match &mut inner.state {
            ConnectionState::Building(building) => if let Poll::Ready(_) = pin!(building).poll(ctx) {
                todo!()
            },

            ConnectionState::Established(established) => if let Poll::Ready(_) = pin!(established).poll(ctx) {
                todo!()
            },

            ConnectionState::Shutdown(_shutdown) => {
                return Poll::Ready(());
            },
        }

        return Poll::Pending;
    }
}