use std::{future::Future, pin::{Pin, pin}, sync::{Arc, Mutex}, task::Poll};
use async_executor::Executor;
use crate::config::*;

pub(crate) fn create(
    executor: Arc<Executor<'static>>,
    auth: ServerAuthentication,
    verify: ClientVerification,
) -> EndpointRef {
    EndpointRef {
        inner: Arc::new(Mutex::new(EndpointInner {
            state: EndpointState::Building(Building {
                auth,
                verify,
            }),
        }))
    }
}

#[derive(Clone)]
pub(crate) struct EndpointRef {
    pub(super) inner: Arc<Mutex<EndpointInner>>,
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
    auth: ServerAuthentication,
    verify: ClientVerification,
}

impl Future for Building {
    type Output = Result<Established, Shutdown>;

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        todo!()
    }
}

struct Established {
    quinn_state: quinn_proto::Endpoint,
}

impl Future for Established {
    type Output = Shutdown;

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        todo!()
    }
}

struct Shutdown {

}

struct EndpointDriver(EndpointRef);

impl Future for EndpointDriver {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let mut inner = self.0.inner.lock().unwrap();

        match &mut inner.state {
            EndpointState::Building(building) => if let Poll::Ready(_) = pin!(building).poll(ctx) {
                todo!()
            },

            EndpointState::Established(established) => if let Poll::Ready(_) = pin!(established).poll(ctx) {
                todo!()
            },

            EndpointState::Shutdown(_shutdown) => {
                return Poll::Ready(());
            },
        }

        return Poll::Pending;
    }
}