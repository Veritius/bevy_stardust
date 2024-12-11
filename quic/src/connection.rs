use std::{future::Future, net::SocketAddr, pin::{pin, Pin}, sync::{Arc, Mutex}, task::{Context, Poll}, time::Instant};
use async_channel::{Receiver, Sender};
use async_task::Task;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use quinn_proto::{ClientConfig, EndpointEvent};
use crate::{endpoint::{ConnectionDgramSender, Endpoint}, events::{C2EEvent, C2EEventSender, E2CEvent}, logging::{LogId, LogIdGen}, taskpool::get_task_pool};

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
#[require(Peer(Peer::new))]
#[require(PeerMessages<Incoming>(PeerMessages::new))]
#[require(PeerMessages<Outgoing>(PeerMessages::new))]
#[require(PeerLifestage(|| PeerLifestage::Handshaking))]
pub struct Connection {
    task: Task<Result<(), ConnectionError>>,
    shared: Arc<Shared>,

    close_signal_tx: Sender<ConnectionCloseSignal>,
    message_incoming_rx: crossbeam_channel::Receiver<ChannelMessage>,
    message_outgoing_tx: Sender<ChannelMessage>,
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Order the task to close, since normally dropping it would detach it, running it to completion in the background.
        // We also don't care about the error case here since it means the connection is already closing or closed.
        let _ = self.close_signal_tx.try_send(ConnectionCloseSignal {

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
        // Create attempt sender/receiver pair and various channels for communication
        let (tx, rx) = async_channel::bounded(1);
        let (close_signal_tx, close_signal_rx) = async_channel::bounded(1);
        let (message_incoming_tx, message_incoming_rx) = crossbeam_channel::unbounded();
        let (message_outgoing_tx, message_outgoing_rx) = async_channel::unbounded();

        // Construct and send the attempt to the endpoint
        endpoint.request_outgoing(OutgoingConnectionAttempt {
            data: ConnectionAttemptData {
                remote_address,
                server_name,
                config,
                close_signal_tx: close_signal_tx.clone(),
            },

            tx,
        });

        // Our future for waiting for the response of the endpoint
        let attempt = ConnectionAttempt { rx };

        let shared = Arc::new(Shared {
            outer_state: Mutex::new(ConnectionState::Connecting),
        });

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
            shared.clone(),
            attempt,
            bundle,
        ));

        // Return component handle thingy
        return Connection {
            task,
            shared,

            close_signal_tx,
            message_incoming_rx,
            message_outgoing_tx,
        };
    }

    pub(crate) fn incoming(
        endpoint: Endpoint,
        data: ConnectionAccepted,
    ) -> (Connection, Sender<ConnectionCloseSignal>) {
        let log_id = LogIdGen::next();

        // Fetch some data before we lose the ability to access it
        let address = data.quinn.remote_address();

        let shared = Arc::new(Shared {
            outer_state: Mutex::new(ConnectionState::Connecting),
        });

        // Channels for communication
        let (close_signal_tx, close_signal_rx) = async_channel::bounded(1);
        let (message_incoming_tx, message_incoming_rx) = crossbeam_channel::unbounded();
        let (message_outgoing_tx, message_outgoing_rx) = async_channel::unbounded();

        // Construct state object
        let state = State {
            log_id: log_id.clone(),
            shared: shared.clone(),
            close_signal_rx,
            endpoint,
            dgram_tx: data.dgram_tx,
            lifestage: Lifestage::Connecting,
            quinn: *data.quinn,
            c2e_event_tx: data.c2e_event_tx,
            e2c_event_rx: data.e2c_event_rx,
            message_incoming_tx,
            message_outgoing_rx,
        };

        // get the task pool so we can spawn tasks
        let task_pool = get_task_pool();

        // Spawn the driver directly since we've already constructed the endpoint
        let task = task_pool.spawn(driver(state));

        // Construct connection handle
        let connection = Connection {
            task,
            shared,

            close_signal_tx: close_signal_tx.clone(),
            message_incoming_rx,
            message_outgoing_tx,
        };

        // Log the creation of the new incoming connection
        // This is separate from outgoing connections because that's behind a future
        log::debug!("Incoming connection {log_id} from address {address} created");

        // Return component handle thingy
        return (connection, close_signal_tx);
    }

    /// Gracefully closes the connection.
    /// 
    /// If the connection is the only holder of an [`Endpoint`] handle,
    /// the endpoint will also shut down shortly after.
    pub fn close(&self) {
        // We send an event to the state object to shut it down.
        // If there's an error, it means the connection is either
        // already closing or closed, so we can safely ignore it.
        let _ = self.close_signal_tx.try_send(ConnectionCloseSignal {

        });
    }

    /// Returns the current state of the connection.
    pub fn state(&self) -> ConnectionState {
        *self.shared.outer_state.lock().unwrap()
    }
}

/// The current state of a [`Connection`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectionState {
    /// The connection is still being established.
    /// Dropping the handle may cause data loss.
    Connecting,

    /// The connection is currently running.
    /// Dropping the handle will cause data loss.
    Connected,

    /// The connection is in the process of shutting down.
    /// Dropping the handle will cause data loss.
    Closing,

    /// The connection has fully shut down and been drained.
    /// Dropping the handle will not cause data loss.
    Closed,
}

impl From<Lifestage> for ConnectionState {
    fn from(value: Lifestage) -> Self {
        match value {
            Lifestage::Connecting => ConnectionState::Connecting,
            Lifestage::Connected => ConnectionState::Connected,
            Lifestage::Closing => ConnectionState::Closing,
            Lifestage::Closed => ConnectionState::Closed,
        }
    }
}

/// An error returned during the creation of a [`Connection`].
#[derive(Debug)]
pub enum ConnectError {
    /// The endpoint this connection relied on was closed.
    EndpointClosed,

    /// An error relating to the QUIC protocol.
    Quic(quinn_proto::ConnectError),
}

/// An error returned during the creation or execution of a [`Connection`].
#[derive(Debug)]
pub enum ConnectionError {
    /// Error returned while creating an endpoint.
    ConnectError(ConnectError),

    /// The endpoint this connection relied on was closed.
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
    pub remote_address: SocketAddr,
    pub server_name: Arc<str>,
    pub config: ClientConfig,

    pub close_signal_tx: Sender<ConnectionCloseSignal>,
}

pub(crate) enum ConnectionAttemptResponse {
    Accepted(ConnectionAccepted),
    Rejected(ConnectionError),
}

pub(crate) struct ConnectionAccepted {
    pub quinn: Box<quinn_proto::Connection>,

    pub c2e_event_tx: C2EEventSender,
    pub e2c_event_rx: Receiver<E2CEvent>,

    pub dgram_tx: ConnectionDgramSender,
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

struct Shared {
    outer_state: Mutex<ConnectionState>
}

struct State {
    shared: Arc<Shared>,
    log_id: LogId,

    close_signal_rx: Receiver<ConnectionCloseSignal>,

    endpoint: Endpoint,

    dgram_tx: ConnectionDgramSender,

    lifestage: Lifestage,

    quinn: quinn_proto::Connection,

    c2e_event_tx: C2EEventSender,
    e2c_event_rx: Receiver<E2CEvent>,

    message_incoming_tx: crossbeam_channel::Sender<ChannelMessage>,
    message_outgoing_rx: Receiver<ChannelMessage>,
}

// Required for the driver future
impl Unpin for State {}

impl Drop for State {
    fn drop(&mut self) {
        // Notify the endpoint of the state object being dropped
        // This is necessary to free up memory in the endpoint
        // Blocking sends should be fine since the channel is unbounded
        let _ = self.c2e_event_tx.send_blocking(C2EEvent::Quinn(
            EndpointEvent::drained()
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifestage {
    Connecting,
    Connected,
    Closing,
    Closed,
}

pub(crate) struct ConnectionCloseSignal {

}

impl ConnectionCloseSignal {
    pub(crate) fn endpoint_shutdown() -> ConnectionCloseSignal {
        ConnectionCloseSignal {
            
        }
    }
}

struct ChannelBundle {
    close_signal_rx: Receiver<ConnectionCloseSignal>,
    message_incoming_tx: crossbeam_channel::Sender<ChannelMessage>,
    message_outgoing_rx: Receiver<ChannelMessage>,
}

async fn build_task(
    endpoint: Endpoint,
    shared: Arc<Shared>,
    attempt: ConnectionAttempt,
    bundle: ChannelBundle,
) -> Result<(), ConnectionError> {
    // Wait for the response of the endpoint
    let accepted = match attempt.await {
        // directly return this and continue with the rest of the code
        ConnectionAttemptResponse::Accepted(response) => response,

        // if we get rejected, we just end the task right here
        ConnectionAttemptResponse::Rejected(response) => {
            return Err(response);
        },
    };

    // Construct the state object
    let state = State {
        shared,
        log_id: LogIdGen::next(),
        close_signal_rx: bundle.close_signal_rx,
        endpoint,
        dgram_tx: accepted.dgram_tx,
        lifestage: Lifestage::Connected,
        quinn: *accepted.quinn,
        c2e_event_tx: accepted.c2e_event_tx,
        e2c_event_rx: accepted.e2c_event_rx,
        message_incoming_tx: bundle.message_incoming_tx,
        message_outgoing_rx: bundle.message_outgoing_rx,
    };

    // Log the acceptance of the connection
    log::trace!("Connection {} was accepted by endpoint {}", state.log_id, state.endpoint.log_id());

    // Run the driver task to completion
    return driver(state).await;
}

async fn driver(
    mut state: State,
) -> Result<(), ConnectionError> {
    use futures_lite::StreamExt;

    enum Event {
        CloseSignal(ConnectionCloseSignal),
        E2CEvent(E2CEvent),
        OutgoingMessage(ChannelMessage),
    }

    let mut stream = pin!({
            let close_signal_rx = state.close_signal_rx.clone().map(|v| Event::CloseSignal(v));
            let e2c_event_rx = state.e2c_event_rx.clone().map(|v| Event::E2CEvent(v));
            let message_outgoing_rx = state.message_outgoing_rx.clone().map(|v| Event::OutgoingMessage(v));

            close_signal_rx
                .or(e2c_event_rx)
                .or(message_outgoing_rx)
    });

    loop {
        let event = match stream.next().await {
            Some(event) => event,
            None => todo!(),
        };

        match event {
            Event::CloseSignal(signal) => handle_close_signal(&mut state, signal),
            Event::E2CEvent(event) => handle_e2c_event(&mut state, event),
            Event::OutgoingMessage(message) => handle_outgoing_message(&mut state, message),
        }
    }
}

fn handle_close_signal(
    state: &mut State,
    signal: ConnectionCloseSignal,
) {
    log::trace!("Received close signal for connection {}", state.log_id);

    state.quinn.close(
        Instant::now(),
        todo!(),
        todo!(),
    );
}

fn handle_e2c_event(
    state: &mut State,
    event: E2CEvent,
) {
    match event {
        E2CEvent::Quinn(event) => state.quinn.handle_event(event),
    }
}

fn handle_outgoing_message(
    state: &mut State,
    message: ChannelMessage,
) {
    todo!()
}