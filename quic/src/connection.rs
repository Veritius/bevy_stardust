use std::{net::SocketAddr, pin::pin, sync::{Arc, Mutex}, time::{Duration, Instant}};
use async_channel::{Receiver, Sender};
use async_io::Timer;
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
    let accepted = match attempt.rx.recv().await {
        // directly return this and continue with the rest of the code
        Ok(ConnectionAttemptResponse::Accepted(response)) => response,

        // if we get rejected, we just end the task right here
        Ok(ConnectionAttemptResponse::Rejected(response)) => {
            return Err(response);
        },

        // If an error occurs, it means the channel was dropped,
        // so we can safely assume that the endpoint is now closed.
        Err(_) => {
            return Err(ConnectionError::EndpointClosed);
        }
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

    // Run the driver task to completion
    return driver(state).await;
}

async fn driver(
    mut state: State,
) -> Result<(), ConnectionError> {
    use futures_lite::StreamExt;

    enum Event {
        DeadlineHit,
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

    let mut scratch = Vec::new();

    loop {
        let timer = match state.poll_timeout() {
            Some(deadline) => Timer::at(deadline),
            None => Timer::after(Duration::from_secs(3)),
        };

        let future = futures_lite::future::race(
            stream.next(),
            async { timer.await; Some(Event::DeadlineHit) },
        );

        let event = match future.await {
            Some(event) => event,
            None => todo!(),
        };

        match event {
            Event::DeadlineHit => {},
            Event::CloseSignal(signal) => state.handle_close_signal(signal),
            Event::E2CEvent(event) => state.handle_e2c_event(event),
            Event::OutgoingMessage(message) => state.handle_outgoing_message(message),
        }

        state.tick(&mut scratch);

        if state.quinn.is_drained() {
            *state.shared.outer_state.lock().unwrap() = ConnectionState::Closed;
            let _ = state.c2e_event_tx.send_blocking(C2EEvent::Quinn(EndpointEvent::drained()));
            return Ok(());
        }
    }
}

// Events
impl State {
    fn handle_close_signal(&mut self, signal: ConnectionCloseSignal) {
        log::trace!("Received close signal for connection {}", self.log_id);

        self.quinn.close(
            Instant::now(),
            todo!(),
            todo!(),
        );
    }

    fn handle_e2c_event(&mut self, event: E2CEvent, ) {
        match event {
            E2CEvent::Quinn(event) => self.quinn.handle_event(event),
        }
    }

    fn handle_outgoing_message(&mut self, message: ChannelMessage) {
        todo!()
    }
}

// Polling stuff
impl State {
    fn tick(&mut self, scratch: &mut Vec<u8>) {
        // Handle timeouts
        self.quinn.handle_timeout(Instant::now());

        // Drain all datagrams into the sending queue
        while let Some(transmit) = self.quinn.poll_transmit(
            Instant::now(),
            1,
            scratch,
        )  {
            let payload = Bytes::copy_from_slice(&scratch[..transmit.size]);
            self.dgram_tx.send(payload);
            scratch.clear();
        }

        // Drain all endpoint events into the channel
        while let Some(event) = self.quinn.poll_endpoint_events() {
            // The channel is unbounded, so it shouldn't error in most cases
            let _ = self.c2e_event_tx.send_blocking(C2EEvent::Quinn(event));
        }

        // Handle all quinn app events
        while let Some(event) = self.quinn.poll() {
            self.handle_quinn_app_event(event);
        }
    }

    #[must_use]
    fn poll_timeout(&mut self) -> Option<Instant> {
        self.quinn.poll_timeout()
    }
}

// Quinn stuff
impl State {
    fn handle_quinn_app_event(&mut self, event: quinn_proto::Event) {
        match event {
            quinn_proto::Event::Stream(event) => self.handle_quinn_stream_event(event),
            quinn_proto::Event::DatagramReceived => self.handle_potential_incoming_dgrams(),
            quinn_proto::Event::DatagramsUnblocked => todo!(),

            quinn_proto::Event::Connected => todo!(),
            quinn_proto::Event::ConnectionLost { reason } => todo!(),
    
            quinn_proto::Event::HandshakeDataReady => { /* Don't care */ },
        }
    }

    fn handle_quinn_stream_event(&mut self, event: quinn_proto::StreamEvent) {
        match event {
            quinn_proto::StreamEvent::Opened { dir } => todo!(),
            quinn_proto::StreamEvent::Readable { id } => todo!(),
            quinn_proto::StreamEvent::Writable { id } => todo!(),
            quinn_proto::StreamEvent::Finished { id } => todo!(),
            quinn_proto::StreamEvent::Stopped { id, error_code } => todo!(),
            quinn_proto::StreamEvent::Available { dir } => todo!(),
        }
    }

    fn handle_potential_incoming_dgrams(&mut self) {
        while let Some(dgram) = self.quinn.datagrams().recv() {
            self.handle_incoming_datagram(dgram);
        }
    }

    fn handle_incoming_datagram(&mut self, dgram: Bytes) {
        todo!()
    }
}