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
            state: EndpointState::Building(Building::new(
                auth,
                verify,
            )),
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
    future: Box<dyn Future<Output = Result<Established, Shutdown>> + Send + Sync>,
}

impl Building {
    fn new(
        auth: ServerAuthentication,
        verify: ClientVerification,
    ) -> Self {
        let future = async {
            let server_config = match auth {
                ServerAuthentication::Authenticated {
                    certificates,
                    private_key,
                } => {
                    let certificates = match certificates.resolve_async().await {
                        Ok(v) => v,
                        Err(_) => todo!(),
                    };

                    let private_key = match private_key.resolve_async().await {
                        Ok(v) => v,
                        Err(_) => todo!(),
                    };

                    Some(match quinn_proto::ServerConfig::with_single_cert(
                        certificates,
                        private_key,
                    ) {
                        Ok(v) => Arc::new(v),
                        Err(_) => todo!(),
                    })
                },

                ServerAuthentication::Disabled => None,
            };

            return Ok(Established {
                quinn_state: quinn_proto::Endpoint::new(
                    Arc::new(quinn_proto::EndpointConfig::default()),
                    server_config,
                    true,
                    None,
                ),
            });
        };

        return Self {
            future: Box::new(future),
        };
    }
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