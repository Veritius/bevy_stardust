use std::{future::Future, net::SocketAddr, pin::{pin, Pin}, sync::Arc, task::{Context, Poll}};
use async_channel::{Receiver, Sender};
use async_task::Task;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use quinn_proto::{ClientConfig, ConnectionHandle};
use crate::{endpoint::Endpoint, events::{C2EEvent, E2CEvent}, taskpool::get_task_pool};

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
    task: Task<Result<(), ConnectionError>>,

    close_signal_tx: Sender<CloseSignal>,
    message_incoming_rx: Receiver<ChannelMessage>,
    message_outgoing_tx: Sender<ChannelMessage>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Order the task to close, since normally dropping it would detach it, running it to completion in the background.
        // We also don't care about the error case here since it means the connection is already closing or closed.
        let _ = self.close_signal_tx.try_send(CloseSignal {

        });
    }
}

impl Connection {
    /// Creates a new outgoing [`Connection`].
    #[must_use]
    pub fn connect(
        endpoint: Endpoint,
        remote_address: SocketAddr,
        server_name: Arc<str>,
        config: ClientConfig,
    ) -> Connection {
        // Create attempt sender/receiver pair
        let (tx, rx) = async_channel::bounded(1);

        // Construct and send the attempt to the endpoint
        endpoint.request_outgoing(OutgoingConnectionAttempt {
            data: ConnectionAttemptData {
                remote_address,
                server_name,
                config,
            },

            tx,
        });

        // Our future for waiting for the response of the endpoint
        let attempt = ConnectionAttempt { rx };

        // Channels for communication
        let (close_signal_tx, close_signal_rx) = async_channel::bounded(1);
        let (message_incoming_tx, message_incoming_rx) = async_channel::unbounded();
        let (message_outgoing_tx, message_outgoing_rx) = async_channel::unbounded();

        // A bundle of channels used to build the connection
        let bundle = ChannelBundle {
            close_signal_rx,
            message_incoming_tx,
            message_outgoing_rx,
        };

        // get the task pool so we can spawn tasks
        let task_pool = get_task_pool();

        // Spawn a task to build and run the endpoint
        let task = task_pool.spawn(build_task(
            endpoint,
            attempt,
            bundle,
        ));

        // Return component handle thingy
        return Connection {
            task,

            close_signal_tx,
            message_incoming_rx,
            message_outgoing_tx,
        };
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
    /// The endpoint this connection relied was closed.
    EndpointClosed,

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

pub(crate) struct OutgoingConnectionAttempt {
    pub data: ConnectionAttemptData,
    pub tx: Sender<ConnectionAttemptResponse>,
}

pub(crate) struct ConnectionAttemptData {
    remote_address: SocketAddr,
    server_name: Arc<str>,
    config: ClientConfig,
}

pub(crate) enum ConnectionAttemptResponse {
    Accepted(ConnectionAccepted),
    Rejected(ConnectionError),
}

pub(crate) struct ConnectionAccepted {
    pub quinn: quinn_proto::Connection,

    pub c2e_event_tx: Sender<(ConnectionHandle, C2EEvent)>,
    pub e2c_event_rx: Receiver<E2CEvent>,
}

struct ConnectionAttempt {
    rx: Receiver<ConnectionAttemptResponse>,
}

impl Future for ConnectionAttempt {
    type Output = ConnectionAttemptResponse;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match pin!(self.rx.recv()).poll(cx) {
            // Response received from sender
            Poll::Ready(Ok(v)) => Poll::Ready(v),

            // Channel is dropped and empty
            // Fabricate a response saying we were rejected
            Poll::Ready(Err(_)) => Poll::Ready(ConnectionAttemptResponse::Rejected(
                ConnectionError::EndpointClosed,
            )),

            // Nothing yet, keep waiting
            Poll::Pending => Poll::Pending,
        }
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

// Needed for the driver future
impl Unpin for State {}

struct CloseSignal {

}

struct ChannelBundle {
    close_signal_rx: Receiver<CloseSignal>,
    message_incoming_tx: Sender<ChannelMessage>,
    message_outgoing_rx: Receiver<ChannelMessage>,
}

async fn build_task(
    endpoint: Endpoint,
    attempt: ConnectionAttempt,
    bundle: ChannelBundle,
) -> Result<(), ConnectionError> {
    // Wait for the response of the endpoint
    let accepted = match attempt.await {
        // directly return this and continue with the rest of the code
        ConnectionAttemptResponse::Accepted(response) => response,

        // if we get rejected, we just end the task right here
        ConnectionAttemptResponse::Rejected(response) => {
            return Err(todo!());
        },
    };

    // Construct the state object
    let state = State {
        close_signal_rx: bundle.close_signal_rx,
        endpoint,
        quinn: accepted.quinn,
        c2e_event_tx: accepted.c2e_event_tx,
        e2c_event_rx: accepted.e2c_event_rx,
        message_incoming_tx: bundle.message_incoming_tx,
        message_outgoing_rx: bundle.message_outgoing_rx,
    };

    // Run the driver task to completion
    return Driver(state).await;
}

struct Driver(State);

impl Future for Driver {
    type Output = Result<(), ConnectionError>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let state = &mut self.0;

        match pin!(state.close_signal_rx.recv()).poll(cx) {
            Poll::Ready(Ok(signal)) => todo!(),
            Poll::Ready(Err(_)) => todo!(),
            Poll::Pending => todo!(),
        }

        match pin!(state.e2c_event_rx.recv()).poll(cx) {
            Poll::Ready(Ok(event)) => todo!(),
            Poll::Ready(Err(_)) => todo!(),
            Poll::Pending => todo!(),
        }

        match pin!(state.message_outgoing_rx.recv()).poll(cx) {
            Poll::Ready(Ok(message)) => todo!(),
            Poll::Ready(Err(_)) => todo!(),
            Poll::Pending => todo!(),
        }

        // We're not done.
        return Poll::Pending;
    }
}