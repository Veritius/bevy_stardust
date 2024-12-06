use std::{future::Future, net::SocketAddr, pin::Pin, sync::Arc, task::{Context, Poll}};
use async_channel::{Receiver, Sender};
use async_task::Task;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::{endpoint::Endpoint, EndpointError};

/// A handle to a QUIC connection.
#[derive(Component)]
pub struct Connection {
    task: Task<()>,
    shared: Arc<Shared>,

    message_incoming_rx: Receiver<ChannelMessage>,
    message_outgoing_tx: Sender<ChannelMessage>,
}

impl Connection {
    /// Creates a new outgoing [`Connection`].
    pub fn connect(
        endpoint: Endpoint,
        remote_address: SocketAddr,
        server_name: Arc<str>,
    ) -> Connection {
        todo!()
    }

    /// Returns the [`Endpoint`] managing this connection.
    pub fn endpoint(&self) -> Endpoint {
        self.shared.endpoint.clone()
    }
}

/// An error returned during the creation or execution of a [`Connection`].
#[derive(Debug)]
pub enum ConnectionError {
    /// The endpoint this connection relied on shut down.
    EndpointError(EndpointError),

    /// The transport protocol was violated.
    TransportError {
        /// The type of error triggered.
        code: u64,
        /// The frame type that triggered the error, if any.
        frame_type: Option<u64>,
        /// Human-readable reason for the error.
        reason: Arc<str>,
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