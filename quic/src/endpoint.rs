use std::{collections::HashMap, future::Future, io::Error as IoError, net::{SocketAddr, ToSocketAddrs, UdpSocket}, pin::{pin, Pin}, sync::{Arc, Mutex, Weak}, time::Instant};
use async_task::Task;
use async_channel::{Receiver, Sender};
use async_io::Async;
use bytes::{Bytes, BytesMut};
use quinn_proto::{ConnectionHandle, DatagramEvent, EndpointEvent};
use crate::{config::{EndpointConfig, ServerConfig}, connection::{ConnectError, ConnectionAccepted, ConnectionAttemptResponse, ConnectionCloseSignal, OutgoingConnectionAttempt}, events::{C2EEvent, C2EEventSender, E2CEvent}, futures::Race, logging::{LogId, LogIdGen}, taskpool::{get_task_pool, NetworkTaskPool}, Connection, ConnectionError};

/// A [`Future`] for the creation of an [`Endpoint`].
/// 
/// This future is automatically run in the background and does not need to be polled by the user.
pub struct LoadingEndpoint(pub(crate) Task<Result<Endpoint, EndpointError>>);

impl Future for LoadingEndpoint {
    type Output = Result<Endpoint, EndpointError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Future::poll(Pin::new(&mut self.0), cx)
    }
}

/// A reference-counted handle to a QUIC endpoint.
/// 
/// Endpoints manage connections and asynchronously handle I/O.
/// 
/// # Reference-counting
/// As long as an instance of this handle exists, the asynchronous endpoint task will be kept alive and running.
/// When all handles are dropped, the endpoint will shut down. Endpoints can be closed early, without dropping
/// all handles, by using [`close`](Self::close). This frees up most resources until all handles are dropped.
/// 
/// If you want to have access to the endpoint without holding it open, see [`downgrade`](Self::downgrade) and [`EndpointWeak`].
#[derive(Clone)]
pub struct Endpoint(Arc<Handle>);

impl Endpoint {
    pub fn new(
        config: EndpointConfig,
        server_config: Option<ServerConfig>,
    ) -> Endpoint {
        Self::new_inner(
            config,
            server_config,
        )
    }

    fn new_inner(
        config: EndpointConfig,
        server_config: Option<ServerConfig>,
    ) -> Endpoint {
        // Retrieve task pool and create a logging id
        let log_id = LogIdGen::next();
        let task_pool = get_task_pool();

        // Unwrapping is fine here because we always bind our sockets
        let address = config.socket.get_ref().local_addr().unwrap();

        // Create channels for communication
        let (io_recv_tx, io_recv_rx) = async_channel::unbounded();
        let (io_send_tx, io_send_rx) = async_channel::unbounded();
        let (conn_event_tx, conn_event_rx) = async_channel::unbounded();
        let (close_signal_tx, close_signal_rx) = async_channel::bounded(1);
        let (outgoing_request_tx, outgoing_request_rx) = async_channel::unbounded();
        let (incoming_connect_tx, incoming_connect_rx) = async_channel::unbounded();

        // we have to do the next steps in a closure because of a cyclic reference
        let handle = Arc::new_cyclic(|handle| {
            // Construct the inner state
            let state = State {
                handle: handle.clone(),
                log_id: log_id.clone(),

                close_signal_rx,
                outgoing_request_rx,
                incoming_connect_tx,

                io_socket: config.socket.clone(),

                io_task: task_pool.spawn(io_task(
                    config.socket,
                    io_recv_tx,
                    io_send_rx
                )),

                io_recv_rx,
                io_send_tx,

                lifestage: Lifestage::Running,

                quinn: quinn_proto::Endpoint::new(
                    config.quinn.clone(),
                    todo!(),
                    true,
                    None,
                ),

                c2e_event_rx: conn_event_rx,
                c2e_event_tx: conn_event_tx,

                connections: HashMap::new(),
            };

            // Start driver task to run in the background
            let driver = task_pool.spawn(driver(state));

            Handle {
                log_id: log_id.clone(),
                driver,

                outer_state: Mutex::new(EndpointState::Running),
                close_signal_tx,
                outgoing_request_tx,
                incoming_connect_rx,
            }
        });

        // Log the creation of the connection
        log::debug!("Endpoint {log_id} successfully created on socket {address}");

        // Return endpoint handle
        return Endpoint(handle);
    }

    /// Gracefully closes all connections and shuts down the endpoint.
    pub fn close(&self) {
        // We send an event to the state object to shut it down.
        // If there's an error, it means the endpoint is either
        // already closing or closed, so we can safely ignore it.
        let _ = self.0.close_signal_tx.send_blocking(EndpointCloseSignal {

        });
    }

    /// Returns the state of the [`Endpoint`].
    pub fn state(&self) -> EndpointState {
        self.0.outer_state.lock().unwrap().clone()
    }

    /// Returns any new incoming connections on the endpoint.
    /// 
    /// This should be called multiple times until it returns `None` once a frame.
    pub fn poll_connections(&self) -> Option<Connection> {
        self.0.incoming_connect_rx.try_recv().ok()
    }

    /// Produces a weak handle ([`EndpointWeak`]), which can still be used to
    /// access the endpoint so long as at least one strong handle exists.
    pub fn downgrade(self) -> EndpointWeak {
        EndpointWeak(Arc::downgrade(&self.0))
    }
}

impl Endpoint {
    pub(crate) fn log_id(&self) -> &LogId {
        &self.0.log_id
    }

    pub(crate) fn request_outgoing(
        &self,
        request: OutgoingConnectionAttempt,
    ) {
        let _ = self.0.outgoing_request_tx.send_blocking(request);
    }
}

impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        let a = Arc::as_ptr(&self.0);
        let b = Arc::as_ptr(&other.0);
        a as usize == b as usize
    }
}

impl Eq for Endpoint {}

/// A weak handle to an [`Endpoint`]. Doesn't prevent the endpoint from being dropped.
#[derive(Clone)]
pub struct EndpointWeak(Weak<Handle>);

impl EndpointWeak {
    /// Attempts to upgrade to a [strong handle](Endpoint).
    /// 
    /// Returns `None` if the endpoint has been dropped.
    pub fn upgrade(self) -> Option<Endpoint> {
        self.0.upgrade().map(|v| Endpoint(v))
    }
}

impl PartialEq for EndpointWeak {
    fn eq(&self, other: &Self) -> bool {
        let a = Weak::as_ptr(&self.0);
        let b = Weak::as_ptr(&other.0);
        a as usize == b as usize
    }
}

impl Eq for EndpointWeak {}

/// The current state of an [`Endpoint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EndpointState {
    /// The endpoint is currently running.
    /// Dropping all handles will cause data loss.
    Running,

    /// The endpoint is in the process of shutting down.
    /// Dropping all handles may cause data loss.
    Closing,

    /// The endpoint has fully shut down and been drained.
    /// Dropping all handles will not cause data loss.
    Closed,
}

impl From<Lifestage> for EndpointState {
    fn from(value: Lifestage) -> Self {
        match value {
            Lifestage::Running => EndpointState::Running,
            Lifestage::Closing => EndpointState::Closing,
            Lifestage::Closed => EndpointState::Closed,
        }
    }
}

/// An error returned during the creation or execution of an [`Endpoint`].
#[derive(Debug)]
pub enum EndpointError {
    /// An I/O error occurred.
    IoError(std::io::Error),

    /// A TLS error occurred.
    TlsError(rustls::Error),
}

impl From<std::io::Error> for EndpointError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

struct Handle {
    log_id: LogId,
    driver: Task<Result<(), EndpointError>>,

    outer_state: Mutex<EndpointState>,
    close_signal_tx: Sender<EndpointCloseSignal>,
    outgoing_request_tx: Sender<OutgoingConnectionAttempt>,
    incoming_connect_rx: Receiver<Connection>,
}

impl Drop for Handle {
    fn drop(&mut self) {
        log::trace!("Handle for endpoint {} dropped", self.log_id);
    }
}

struct State {
    handle: Weak<Handle>,
    log_id: LogId,

    close_signal_rx: Receiver<EndpointCloseSignal>,
    outgoing_request_rx: Receiver<OutgoingConnectionAttempt>,
    incoming_connect_tx: Sender<Connection>,

    io_socket: Arc<Async<UdpSocket>>,
    io_task: Task<Result<(), std::io::Error>>,

    io_recv_rx: Receiver<DgramRecv>,
    io_send_tx: Sender<DgramSend>,

    lifestage: Lifestage,

    quinn: quinn_proto::Endpoint,

    c2e_event_rx: Receiver<(ConnectionHandle, C2EEvent)>,
    c2e_event_tx: Sender<(ConnectionHandle, C2EEvent)>,

    connections: HashMap<ConnectionHandle, HeldConnection>,
}

// Required for the driver future
impl Unpin for State {}

impl Drop for State {
    fn drop(&mut self) {
        match self.lifestage {
            Lifestage::Running | Lifestage::Closing => log::warn!("Endpoint {} was dropped without being closed", self.log_id),
            Lifestage::Closed => log::trace!("Endpoint {} dropped", self.log_id),
        }

        self.update_lifestage(Lifestage::Closed);
    }
}

impl State {
    fn update_lifestage(&mut self, lifestage: Lifestage) {
        // Update inner lifestage value
        self.lifestage = lifestage;

        // Try to update the state value in the endpoint handle
        // If this fails it's irrelevant since it means it's no longer
        // readable by anyone and that the state object is about to be dropped.
        if let Some(handle) = self.handle.upgrade() {
            *handle.outer_state.lock().unwrap() = lifestage.into();
        }
    }

    fn remove_connection(&mut self, handle: ConnectionHandle) {
        self.quinn.handle_event(handle, EndpointEvent::drained());
        self.connections.remove(&handle);
    }

    fn send_e2c_event(&self, handle: ConnectionHandle, event: E2CEvent) {
        // Retrieve the connection from the map so we can use its channels
        // This shouldn't panic as long as we clean up after ourselves well
        let connection = self.connections.get(&handle).unwrap();

        // This channel is unbounded, so it should be fine to just do a blocking send
        let _ = connection.e2c_event_tx.send_blocking(event);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lifestage {
    Running,
    Closing,
    Closed,
}

struct HeldConnection {
    e2c_event_tx: Sender<E2CEvent>,
    close_signal_tx: Sender<ConnectionCloseSignal>,
}

struct EndpointCloseSignal {

}

struct DgramRecv {
    origin: SocketAddr,
    payload: BytesMut,
}

struct DgramSend {
    destination: SocketAddr,
    payload: Bytes,
}

pub(crate) struct ConnectionDgramSender {
    address: SocketAddr,
    sender: Sender<DgramSend>,
}

impl ConnectionDgramSender {
    pub fn send(&self, payload: Bytes) {
        // Blocking sends should be fine since the channel is unbounded
        let _ = self.sender.send_blocking(DgramSend {
            destination: self.address,
            payload,
        });
    }
}

async fn io_task(
    socket: Arc<Async<UdpSocket>>,
    io_recv_tx: Sender<DgramRecv>,
    io_send_rx: Receiver<DgramSend>,
) -> Result<(), std::io::Error> {
    // TODO: Make this configurable
    let mut scratch = vec![0u8; 2048];

    loop {
        let socket_poller = async {
            match socket.recv_from(&mut scratch[..]).await {
                Ok((length, origin)) => match io_recv_tx.send(DgramRecv {
                    origin,
                    payload: {
                        let mut buf = BytesMut::with_capacity(length);
                        buf.extend_from_slice(&scratch[..length]);
                        buf
                    },
                }).await {
                    Ok(_) => { /* Do nothing */ },
                    Err(_) => todo!(),
                },

                Err(_) => todo!(),
            }
        };

        let send_poller = async {
            match io_send_rx.recv().await {
                Ok(dgram) => match socket.send_to(
                    &dgram.payload,
                    dgram.destination,
                ).await {
                    Ok(_) => { /* Do nothing */ }
                    Err(_) => todo!(),
                },

                Err(_) => todo!(),
            }
        };

        Race::new((
            pin!(socket_poller),
            pin!(send_poller),
        )).await;
    }
}

async fn driver(
    mut state: State,
) -> Result<(), EndpointError> {
    use futures_lite::StreamExt;

    enum Event {
        CloseSignal(EndpointCloseSignal),
        C2EEvent(ConnectionHandle, C2EEvent),
        DgramRecv(DgramRecv),
        OutgoingAttempt(OutgoingConnectionAttempt),
    }

    let mut stream = pin!({
        let close_signal_rx = state.close_signal_rx.clone().map(|v| Event::CloseSignal(v));
        let c2e_event_rx = state.c2e_event_rx.clone().map(|v| Event::C2EEvent(v.0, v.1));
        let dgram_recv_rx = state.io_recv_rx.clone().map(|v| Event::DgramRecv(v));
        let outgoing_request_rx = state.outgoing_request_rx.clone().map(|v| Event::OutgoingAttempt(v));

        close_signal_rx
            .or(c2e_event_rx)
            .or(dgram_recv_rx)
            .or(outgoing_request_rx)
    });

    loop {
        let event = match stream.next().await {
            Some(event) => event,
            None => todo!(),
        };

        match event {
            Event::CloseSignal(signal) => handle_close_signal(&mut state, signal),
            Event::C2EEvent(handle, event) => handle_c2e_event(&mut state, handle, event),
            Event::DgramRecv(dgram) => handle_dgram_recv(&mut state, dgram),
            Event::OutgoingAttempt(attempt) => handle_out_attempt(&mut state, attempt),
        }
    }
}

fn handle_close_signal(
    state: &mut State,
    signal: EndpointCloseSignal,
) {
    match state.lifestage {
        // If we're running, we can close.
        Lifestage::Running => {},

        // If we're already closing/closed, ignore this signal and early return.
        // If we go through it'll probably just muck everything up.
        Lifestage::Closing | Lifestage::Closed => { return },
    }

    // Update lifestage/visible outer state to closing
    state.update_lifestage(Lifestage::Closing);

    // Iterate over all connections and signal them to close
    for (_, connection) in state.connections.iter() {
        // We're fine to ignore any errors from this method, since if it does happen,
        // it means the connection has already been signalled to shut down before this point.
        let _ = connection.close_signal_tx.try_send(ConnectionCloseSignal::endpoint_shutdown());
    }

    // Log the close signal
    log::trace!("Endpoint {} received close signal", state.log_id);
}

fn handle_c2e_event(
    state: &mut State,
    handle: ConnectionHandle,
    event: C2EEvent,
) {
    match event {
        // Quinn events are given directly to the inner endpoint state for handling
        C2EEvent::Quinn(event) => {
            if let Some(event) = state.quinn.handle_event(
                handle,
                event,
            ) {
                // Send the response event to the connection
                state.send_e2c_event(handle, E2CEvent::Quinn(event));
            };
        },
    }
}

fn handle_dgram_recv(
    state: &mut State,
    dgram: DgramRecv,
) {
    let mut scratch = Vec::new();

    match state.quinn.handle(
        Instant::now(),
        dgram.origin,
        None, // TODO: Figure out what this does
        None, // TODO: ECN with async_io/async_net
        dgram.payload,
        &mut scratch,
    ) {
        // Connection event intended for a connection we're taking care of
        Some(DatagramEvent::ConnectionEvent(handle, event)) => {
            state.send_e2c_event(handle, E2CEvent::Quinn(event));
        },

        // An incoming connection can be made
        Some(DatagramEvent::NewConnection(incoming)) => {
            // Extract some data we use later on
            // We do this here because it's inaccessible by the time we want it
            let address = incoming.remote_address();

            // Try to accept the connection
            match state.quinn.accept(
                incoming,
                Instant::now(),
                &mut scratch,
                None,
            ) {
                Ok((handle, quinn)) => {
                    // We handle the case for all strong handles being dropped as
                    // technically this State object is owned by the task, and the
                    // task is owned by the handle we're trying to upgrade. There is
                    // a period when the handle is dropped before this type is dropped,
                    // therefore we cannot guarantee that we won't panic when we try to
                    // unwrap the handle.
                    let endpoint_handle = match state.handle.upgrade() {
                        Some(v) => v,
                        None => { return },
                    };

                    // Construct channels for exchanging messages
                    let c2e_event_tx = C2EEventSender::new(handle, state.c2e_event_tx.clone());
                    let (e2c_event_tx, e2c_event_rx) = async_channel::unbounded();

                    // Construct connection
                    let (connection, close_signal_tx) = Connection::incoming(
                        Endpoint(endpoint_handle),
                        ConnectionAccepted {
                            quinn: Box::new(quinn),
                            c2e_event_tx,
                            e2c_event_rx,
                            dgram_tx: ConnectionDgramSender {
                                address,
                                sender: state.io_send_tx.clone(),
                            },
                        },
                    );

                    // Add connection to the map
                    state.connections.insert(handle, HeldConnection {
                        e2c_event_tx,
                        close_signal_tx,
                    });

                    // Throw the connection into the queue for the user to pick up
                    // Blocking send is fine since the channel is unbounded
                    // We can discard the result because we know that the receiver
                    // exists, because we're holding a reference to it right now.
                    let _ = state.incoming_connect_tx.send_blocking(connection);

                    // Log the beginning of the connection
                    log::debug!("Incoming connection supposedly from {address} accepted on endpoint {}", state.log_id);
                },

                Err(err) => {
                    // Send the response packet if one is included
                    if let Some(transmit) = err.response {
                        // Blocking sends should be fine since the channel is unbounded
                        let _ = state.io_send_tx.send_blocking(DgramSend {
                            destination: transmit.destination,
                            payload: Bytes::copy_from_slice(&scratch[..transmit.size])
                        });
                    }

                    // Log the failure to connect, useful for debugging
                    log::debug!("Incoming connection supposedly from {address} rejected from endpoint {}: {}", state.log_id, err.cause);
                },
            }
        },

        // Endpoint wants to send a datagram, no strings attached
        Some(DatagramEvent::Response(transmit)) => {
            // Blocking sends should be fine since the channel is unbounded
            let _ = state.io_send_tx.send_blocking(DgramSend {
                destination: transmit.destination,
                payload: Bytes::copy_from_slice(&scratch[..transmit.size])
            });
        }

        // No side effects.
        None => {},
    }
}

fn handle_out_attempt(
    state: &mut State,
    attempt: OutgoingConnectionAttempt,
) {
    // Try to create a connection through Quinn first
    let (handle, quinn) = match state.quinn.connect(
        Instant::now(),
        attempt.data.config.quinn,
        attempt.data.config.remote_address,
        &attempt.data.config.server_name,
    ) {
        Ok(v) => v,
        Err(err) => {
            // Construct friendly error message
            let err = ConnectionError::ConnectError(ConnectError::Quic(err));

            // Log the rejection for debugging purposes
            // log::debug!("Outgoing connection {} to {} rejected by endpoint {}: {err:?}",
            //     state.log_id, attempt.data.config.remote_address);

            // Message handling to notify the receiver so it can be dropped
            let _ = attempt.tx.send_blocking(ConnectionAttemptResponse::Rejected(err));

            // Done
            return;
        },
    };

    // Extract some data from the connection before we lose access
    let address = quinn.remote_address();

    // Construct channels for exchanging messages
    let c2e_event_tx = C2EEventSender::new(handle, state.c2e_event_tx.clone());
    let (e2c_event_tx, e2c_event_rx) = async_channel::unbounded();

    // Try to notify the receiver that they've been accepted
    // Sending messages can fail if the receiver is dropped,
    // so we have to handle that case. Also, funny closure magic
    // so we can use the ? operator. It probably gets optimised out
    // so I don't really care. One day we'll have try blocks...
    if (|| -> Result<(), ()> {
        // Notify the sender. Blocking sends should be fine since the channel is only filled here.
        attempt.tx.send_blocking(ConnectionAttemptResponse::Accepted(ConnectionAccepted {
            quinn: Box::new(quinn),
            c2e_event_tx,
            e2c_event_rx,
            dgram_tx: ConnectionDgramSender {
                address,
                sender: state.io_send_tx.clone(),
            },
        })).map_err(|_| ())?;

        // all messages successfully sent
        return Ok(());
    })().is_err() {
        // We have to do some cleanup here, like letting the endpoint know that the connection has now been removed.
        // We don't use State::remove_connection as that does unnecessary work, since we haven't fully added the connection.
        state.quinn.handle_event(handle, EndpointEvent::drained());

        // Log the request being denied
        log::debug!("Outgoing connection from {address} was almost accepted by endpoint {} but the handle was dropped", state.log_id);

        // All done.
        return;
    };

    // Add connection to the map
    state.connections.insert(handle, HeldConnection {
        e2c_event_tx,
        close_signal_tx: attempt.data.close_signal_tx,
    });

    // Log the connection being accepted
    log::debug!("Outgoing connection from {address} was accepted by endpoint {}", state.log_id);
}