use std::{collections::HashMap, future::Future, io::ErrorKind, net::{SocketAddr, ToSocketAddrs}, sync::Arc, task::Poll, time::Instant};
use async_task::Task;
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use bytes::BytesMut;
use crossbeam_channel::TryRecvError;
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnConnectionId, EndpointConfig};
use crate::{channels::mpsc, runtime::Handle as RuntimeHandle, socket::{DgramRecv, DgramSend}};
use crate::{channels::watch, connection::{ConnectionError, ConnectionRequest, NewConnection}, socket::Socket};

/// A builder for the [`Endpoint`] component, using the [typestate] pattern.
/// 
/// [typestate]: http://cliffle.com/blog/rust-typestate/
pub struct EndpointBuilder<T = ()> {
    p: T,
}

impl EndpointBuilder<()> {
    /// Creates a new builder.
    pub fn new() -> EndpointBuilder<WantsRuntime> {
        EndpointBuilder {
            p: WantsRuntime {
                _p: (),
            },
        }
    }
}

/// Step where a runtime is added.
pub struct WantsRuntime {
    _p: (),
}

impl EndpointBuilder<WantsRuntime> {
    /// Uses a runtime.
    pub fn with_runtime(self, runtime: RuntimeHandle) -> EndpointBuilder<WantsSocket> {
        return EndpointBuilder { p: WantsSocket { runtime } };
    }
}

/// Step where a socket is added.
pub struct WantsSocket {
    runtime: RuntimeHandle,
}

impl EndpointBuilder<WantsSocket> {
    /// Binds to a new UDP socket.
    pub fn bind(self, address: impl ToSocketAddrs) -> Result<EndpointBuilder<WantsConfig>, std::io::Error> {
        let socket = Socket::new(address)?;

        return Ok(EndpointBuilder {
            p: WantsConfig {
                runtime: self.p.runtime,
                socket,
            }
        })
    }
}

/// Step where config is added.
pub struct WantsConfig {
    runtime: RuntimeHandle,
    socket: Socket,
}

impl EndpointBuilder<WantsConfig> {
    /// Adds endpoint config.
    pub fn with_config(self, config: impl Into<Arc<EndpointConfig>>) -> EndpointBuilder<MaybeServer> {
        return EndpointBuilder { p: MaybeServer {
            runtime: self.p.runtime,
            socket: self.p.socket,
            config: config.into(),
        } };
    }
}

/// Step where the endpoint may become a server.
pub struct MaybeServer {
    runtime: RuntimeHandle,
    socket: Socket,
    config: Arc<quinn_proto::EndpointConfig>,
}

impl EndpointBuilder<MaybeServer> {
    /// Act as a client.
    pub fn client(self) -> Endpoint {
        open(
            self.p.runtime,
            self.p.socket,
            self.p.config,
            None,
        )
    }

    /// Act as a server.
    pub fn server(
        self,
        server: Arc<quinn_proto::ServerConfig>,
    ) -> Endpoint {
        open(
            self.p.runtime,
            self.p.socket,
            self.p.config,
            Some(server),
        )
    }
}

/// An existing handle to an endpoint.
/// 
/// This component can be transferred freely between entities.
/// When dropped, the endpoint (and all related components) will be dropped.
pub struct Endpoint {
    pub(crate) handle: Handle,

    driver: Task<()>,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

struct State {
    state: watch::Sender<EndpointState>,

    socket: Socket,

    quinn: quinn_proto::Endpoint,

    quinn_event_rx: mpsc::Receiver<EndpointEvent>,
    quinn_event_tx: mpsc::Sender<EndpointEvent>,

    connections: HashMap<QuinnConnectionId, ConnectionHandle>,

    connection_request_rx: mpsc::Receiver<ConnectionRequest>,
    connection_accepted_tx: mpsc::Sender<NewConnection>,
}

pub(crate) struct Handle {
    state: watch::Receiver<EndpointState>,

    connection_request_tx: mpsc::Sender<ConnectionRequest>,
    connection_accepted_rx: mpsc::Receiver<NewConnection>,
}

impl Handle {
    pub fn connect(&self, request: ConnectionRequest) -> Result<(), ConnectionError> {
        self.connection_request_tx.send(request)
            .map_err(|_| ConnectionError::EndpointClosed)
    }
}

pub enum EndpointState {
    Building,
    Established,
    Shutdown,
}

pub(crate) struct EndpointEvent {
    pub id: quinn_proto::ConnectionHandle,
    pub data: quinn_proto::EndpointEvent,
}

struct ConnectionHandle {
    quinn_event_tx: mpsc::Sender<ConnectionEvent>,
}

pub(crate) struct EndpointHandle {
    pub quinn_event_tx: mpsc::Sender<EndpointEvent>,
    pub quinn_event_rx: mpsc::Receiver<ConnectionEvent>,

    _hidden: (),
}

enum BuildError {
    IoError(std::io::Error),
    TlsError(rustls::Error),
}

impl From<std::io::Error> for BuildError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<rustls::Error> for BuildError {
    fn from(value: rustls::Error) -> Self {
        Self::TlsError(value)
    }
}

pub(crate) fn open(
    runtime: RuntimeHandle,
    socket: Socket,
    config: Arc<quinn_proto::EndpointConfig>,
    server: Option<Arc<quinn_proto::ServerConfig>>,
) -> Endpoint {
    // Create various communication channels
    let (state_tx, state_rx) = watch::channel(EndpointState::Building);
    let (connection_request_tx, connection_request_rx) = mpsc::channel();
    let (connection_accepted_tx, connection_accepted_rx) = mpsc::channel();

    Endpoint {
        handle: Handle {
            state: state_rx,

            connection_request_tx,
            connection_accepted_rx,
        },

        driver: runtime.spawn(endpoint(BuildTaskData {
            runtime: runtime.clone(),
            socket,
            config,
            server,

            state_tx,
            connection_request_rx,
            connection_accepted_tx,
        })),
    }
}

async fn endpoint(
    config: BuildTaskData,
) {
    // Try to build endpoint
    let state = match build(config).await {
        Ok(state) => state,
        Err(err) => todo!(),
    };

    log::debug!("Opened endpoint on address {}", state.socket.local_addr());

    EndpointDriver(state).await;
}

struct EndpointDriver(State);

impl Future for EndpointDriver {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut state = &mut self.0;

        while let Ok(dgram) = state.socket.dgram_rx.try_recv() {
            handle_datagram(&mut state, dgram);
        };

        while let Ok(event) = state.quinn_event_rx.try_recv() {
            handle_event(&mut state, event);
        };

        while let Ok(request) = state.connection_request_rx.try_recv() {
            handle_connection_request(&mut state, request);
        }

        return Poll::Pending;
    }
}

struct BuildTaskData {
    runtime: RuntimeHandle,
    socket: Socket,
    config: Arc<quinn_proto::EndpointConfig>,
    server: Option<Arc<quinn_proto::ServerConfig>>,

    state_tx: watch::Sender<EndpointState>,
    connection_request_rx: mpsc::Receiver<ConnectionRequest>,
    connection_accepted_tx: mpsc::Sender<NewConnection>,
}

async fn build(
    config: BuildTaskData,
) -> Result<State, BuildError> {
    // Create communications channels
    let (quinn_event_tx, quinn_event_rx) = mpsc::channel();

    let quinn = quinn_proto::Endpoint::new(
        config.config,
        config.server,
        true,
        None,
    );

    // Return state object
    return Ok(State {
        state: config.state_tx,
        socket: config.socket,
        quinn,
        quinn_event_rx,
        quinn_event_tx,
        connections: HashMap::new(),
        connection_request_rx: config.connection_request_rx,
        connection_accepted_tx: config.connection_accepted_tx,
    });
}

fn handle_datagram(
    state: &mut State,
    dgram: DgramRecv,
) {
    let mut scratch = Vec::new();

    if let Some(response) = state.quinn.handle(
        Instant::now(),
        dgram.origin,
        None,
        None,
        dgram.payload,
        &mut scratch,
    ) {
        match response {
            quinn_proto::DatagramEvent::ConnectionEvent(
                id,
                event,
            ) => {
                let handle = state.connections.get(&id).unwrap();
                handle.quinn_event_tx.send(event).unwrap(); // TODO: Handle error
            },

            quinn_proto::DatagramEvent::NewConnection(incoming) => {
                match state.quinn.accept(
                    incoming,
                    Instant::now(),
                    &mut scratch,
                    None,
                ) {
                    Ok((id, quinn)) => {
                        let connection = add_connection(
                            state,
                            id,
                            quinn,
                        );

                        state.connection_accepted_tx
                            .send(connection)
                            .unwrap(); // TODO: Handle error
                    },

                    Err(_) => todo!(),
                }
            },

            quinn_proto::DatagramEvent::Response(transmit) => {
                let mut payload = BytesMut::with_capacity(transmit.size);
                payload.copy_from_slice(&scratch[..transmit.size]);

                state.socket.dgram_tx.send(DgramSend {
                    target: transmit.destination,
                    payload,
                }).unwrap(); // TODO: Handle error
            },
        }
    }
}

fn handle_event(
    state: &mut State,
    event: EndpointEvent,
) {
    if let Some(response) = state.quinn.handle_event(event.id, event.data) {
        let handle = state.connections.get(&event.id).unwrap();
        handle.quinn_event_tx.send(response).unwrap(); // TODO: Handle error
    }
}

fn handle_connection_request(
    state: &mut State,
    request: ConnectionRequest,
) {
    match state.quinn.connect(
        Instant::now(),
        request.data.client_config,
        request.data.address,
        &request.data.server_name,
    ) {
        Ok((id, quinn)) => {
            request.inner.accept(add_connection(
                state,
                id,
                quinn,
            ));
        },

        Err(err) => {
            request.inner.reject(ConnectionError::QuicError(err));
        },
    }
}

fn add_connection(
    state: &mut State,
    id: QuinnConnectionId,
    quinn: quinn_proto::Connection,
) -> NewConnection {
    log::debug!("Connection established with {}", quinn.remote_address());

    let (quinn_event_tx, quinn_event_rx) = mpsc::channel();

    state.connections.insert(id, ConnectionHandle {
        quinn_event_tx,
    });

    return NewConnection {
        quinn,

        endpoint: EndpointHandle {
            quinn_event_rx,
            quinn_event_tx: state.quinn_event_tx.clone(),

            _hidden: (),
        },
    };
}