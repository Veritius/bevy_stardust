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
        self: Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        todo!()
    }
}

struct Established {
    quinn_state: quinn_proto::Connection,
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

struct ConnectionDriver(ConnectionRef);

impl Future for ConnectionDriver {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let mut inner = self.0.inner.lock().unwrap();

        let keep_going = false;

        match &mut inner.state {
            ConnectionState::Building(building) => if let Poll::Ready(output) = pin!(building).poll(ctx) {
                inner.state = match output {
                    Ok(established) => ConnectionState::Established(established),
                    Err(shutdown) => ConnectionState::Shutdown(shutdown),
                };
            },

            ConnectionState::Established(established) => if let Poll::Ready(output) = pin!(established).poll(ctx) {
                inner.state = ConnectionState::Shutdown(output);
            },

            ConnectionState::Shutdown(_shutdown) => {
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