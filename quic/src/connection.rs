use std::{net::SocketAddr, pin::pin, sync::Arc};
use async_channel::{Receiver, Sender};
use async_task::Task;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use quinn_proto::ConnectionHandle;
use crate::{endpoint::Endpoint, events::{C2EEvent, E2CEvent}, futures::Race, EndpointError};

/// A unique handle to a QUIC connection.
/// 
/// # Endpoints
/// A `Connection` is associated with an [`Endpoint`],
/// usually one it was created with through [`connect`](Self::connect).
/// Endpoints manage I/O for multiple connections.
/// 
/// Holds a shared handle to the [`Endpoint`] it was created with.
/// As a result, a connection will keep its endpoint open for as long as it lives.
/// When the connection finishes, the handle is dropped, even if this type still exists.
#[derive(Component)]
pub struct Connection {
    task: Task<()>,

    close_signal_tx: Sender<CloseSignal>,
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

    /// Gracefully closes the connection.
    /// 
    /// If the connection is the only holder of an [`Endpoint`] handle,
    /// the endpoint will also shut down shortly after.
    pub fn close(&self) {
        // We send an event to the state object to shut it down.
        // If there's an error, it means the connection is either
        // already closing or closed, so we can safely ignore it.
        let _ = self.close_signal_tx.try_send(CloseSignal {

        });
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

struct State {
    close_signal_rx: Receiver<CloseSignal>,

    endpoint: Endpoint,

    quinn: quinn_proto::Connection,

    c2e_event_tx: Sender<(ConnectionHandle, C2EEvent)>,
    e2c_event_rx: Receiver<E2CEvent>,

    message_incoming_tx: Sender<ChannelMessage>,
    message_outgoing_rx: Receiver<ChannelMessage>,
}

struct CloseSignal {

}


async fn driver_task(
    state: State,
) -> EndpointError {
    loop {
        let close_signal = async {
            match state.close_signal_rx.recv().await {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            };
        };

        let event_recv = async {
            match state.e2c_event_rx.recv().await {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            };
        };

        let msg_recv = async {
            match state.message_outgoing_rx.recv().await {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            };
        };

        Race::new((
            pin!(close_signal),
            pin!(event_recv),
            pin!(msg_recv),
        )).await;
    }
}