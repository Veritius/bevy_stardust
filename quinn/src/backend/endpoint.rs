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

        let keep_going = false;

        match &mut inner.state {
            EndpointState::Building(building) => if let Poll::Ready(output) = pin!(building).poll(ctx) {
                inner.state = match output {
                    Ok(established) => EndpointState::Established(established),
                    Err(shutdown) => EndpointState::Shutdown(shutdown),
                };
            },

            EndpointState::Established(established) => if let Poll::Ready(output) = pin!(established).poll(ctx) {
                inner.state = EndpointState::Shutdown(output);
            },

            EndpointState::Shutdown(_shutdown) => {
                return Poll::Ready(());
            },
        }

        // Release lock
        drop(inner);

        if keep_going {
            ctx.waker().wake_by_ref();
        }

        return Poll::Pending;
    }
}