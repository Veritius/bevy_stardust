use std::{future::Future, sync::{Arc, Mutex}};
use async_executor::Executor;
use bevy_stardust::prelude::ChannelMessage;
use crossbeam_channel::{Receiver, Sender};
use crate::config::*;
use super::endpoint::EndpointRef;

pub(crate) fn create(
    executor: Arc<Executor<'static>>,
    endpoint: EndpointRef,
    root_certs: TrustAnchorStoreOrigin,
    server_name: Arc<str>,
) -> ConnectionRef {
    todo!()
}

#[derive(Clone)]
pub(crate) struct ConnectionRef {
    inner: Arc<Mutex<ConnectionInner>>,
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

struct Established {
    quinn_state: quinn_proto::Connection,

    incoming: Sender<ChannelMessage>,
    outgoing: Receiver<ChannelMessage>,
}

struct Shutdown {

}

pub(crate) struct MessageHandle {
    incoming: Receiver<ChannelMessage>,
    outgoing: Sender<ChannelMessage>,
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