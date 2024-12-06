use std::{future::Future, net::SocketAddr, pin::Pin, sync::Arc, task::{Context, Poll}};
use async_channel::{Receiver, Sender};
use async_task::Task;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::endpoint::Endpoint;

/// A handle to a connection.
#[derive(Component)]
pub struct Connection {
    task: Task<()>,
    shared: Arc<Shared>,

    message_incoming_rx: Receiver<ChannelMessage>,
    message_outgoing_tx: Sender<ChannelMessage>,
}

impl Connection {
    pub fn connect(
        endpoint: Endpoint,
        remote_address: SocketAddr,
        server_name: Arc<str>,
    ) -> Connection {
        // let (message_incoming_tx, message_incoming_rx) = async_channel::unbounded();
        // let (message_outgoing_tx, message_outgoing_rx) = async_channel::unbounded();

        todo!()
    }

    /// Returns the [`Endpoint`] managing this connection.
    pub fn endpoint(&self) -> Endpoint {
        self.shared.endpoint.clone()
    }
}

struct Shared {
    endpoint: Endpoint,
}

struct State {
    message_incoming_tx: Sender<ChannelMessage>,
    message_outgoing_rx: Receiver<ChannelMessage>,
}

struct Driver {
    state: State,
}

impl Future for Driver {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        todo!()
    }
}