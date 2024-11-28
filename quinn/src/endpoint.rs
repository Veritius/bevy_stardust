use std::{collections::HashMap, io::ErrorKind, net::SocketAddr, sync::Arc, time::Instant};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use bytes::BytesMut;
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnConnectionId};
use tokio::{net::UdpSocket, runtime::Handle as RuntimeHandle, select, sync::{mpsc, watch, Notify}, task::JoinHandle};
use crate::{commands::MakeEndpointInner, connection::{ConnectionError, ConnectionRequest, NewConnection}};

pub struct Endpoint {
    pub(crate) handle: Handle,

    driver: JoinHandle<()>,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

struct State {
    state: watch::Sender<EndpointState>,

    socket: Arc<UdpSocket>,

    socket_dgram_recv_rx: mpsc::UnboundedReceiver<DatagramRecv>,
    socket_dgram_send_tx: mpsc::UnboundedSender<DatagramSend>,

    quinn: quinn_proto::Endpoint,

    quinn_event_rx: mpsc::UnboundedReceiver<EndpointEvent>,
    quinn_event_tx: mpsc::UnboundedSender<EndpointEvent>,

    connections: HashMap<QuinnConnectionId, ConnectionHandle>,

    connection_request_rx: mpsc::UnboundedReceiver<ConnectionRequest>,
    connection_accepted_tx: mpsc::UnboundedSender<NewConnection>,
}

pub(crate) struct Handle {
    state: watch::Receiver<EndpointState>,

    connection_request_tx: mpsc::UnboundedSender<ConnectionRequest>,
    connection_accepted_rx: mpsc::UnboundedReceiver<NewConnection>,
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
    quinn_event_tx: mpsc::UnboundedSender<ConnectionEvent>,
}

pub(crate) struct EndpointHandle {
    quinn_event_tx: mpsc::UnboundedSender<EndpointEvent>,
    quinn_event_rx: mpsc::UnboundedReceiver<ConnectionEvent>,
}

struct DatagramRecv {
    origin: SocketAddr,
    payload: BytesMut,
}

struct DatagramSend {
    target: SocketAddr,
    payload: BytesMut,
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
    config: MakeEndpointInner,
) -> Endpoint {
    // Create various communication channels
    let (state_tx, state_rx) = tokio::sync::watch::channel(EndpointState::Building);
    let (connection_request_tx, connection_request_rx) = mpsc::unbounded_channel();
    let (connection_accepted_tx, connection_accepted_rx) = mpsc::unbounded_channel();

    Endpoint {
        handle: Handle {
            state: state_rx,

            connection_request_tx,
            connection_accepted_rx,
        },

        driver: runtime.spawn(endpoint(BuildTaskData {
            runtime: runtime.clone(),
            config,

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
    let mut state = match build(config).await {
        Ok(state) => state,
        Err(err) => todo!(),
    };

    loop {
        select! {
            // Receive datagrams
            dgram = state.socket_dgram_recv_rx.recv() => match dgram {
                Some(dgram) => handle_datagram(&mut state, dgram).await,
                None => todo!(),
            },

            // Handle events
            event = state.quinn_event_rx.recv() => match event {
                Some(event) => handle_event(&mut state, event).await,
                None => todo!(),
            },

            // Handle connection requests
            request = state.connection_request_rx.recv() => match request {
                Some(request) => handle_connection_request(&mut state, request).await,
                None => todo!(),
            }
        }
    }
}

struct BuildTaskData {
    runtime: RuntimeHandle,
    config: MakeEndpointInner,

    state_tx: watch::Sender<EndpointState>,
    connection_request_rx: mpsc::UnboundedReceiver<ConnectionRequest>,
    connection_accepted_tx: mpsc::UnboundedSender<NewConnection>,
}

async fn build(
    config: BuildTaskData,
) -> Result<State, BuildError> {
    // Create communications channels
    let (quinn_event_tx, quinn_event_rx) = mpsc::unbounded_channel();
    let (socket_dgram_recv_tx, socket_dgram_recv_rx) = mpsc::unbounded_channel();
    let (socket_dgram_send_tx, socket_dgram_send_rx) = mpsc::unbounded_channel();

    // Resolve user configuration
    let (socket, quinn) = match config.config {
        MakeEndpointInner::Preconfigured {
            socket,
            config,
            server,
        } => {
            socket.set_nonblocking(true)?;
            let socket = UdpSocket::from_std(socket).map_err(|e| BuildError::IoError(e))?;

            let quinn = quinn_proto::Endpoint::new(
                config,
                server,
                true,
                None,
            );

            (Arc::new(socket), quinn)
        },
    };

    // Spawn tasks for I/O
    config.runtime.spawn(io_recv_task(socket.clone(), socket_dgram_recv_tx));
    config.runtime.spawn(io_send_task(socket.clone(), socket_dgram_send_rx));

    // Return state object
    return Ok(State {
        state: config.state_tx,
        socket,
        socket_dgram_recv_rx,
        socket_dgram_send_tx,
        quinn,
        quinn_event_rx,
        quinn_event_tx,
        connections: HashMap::new(),
        connection_request_rx: config.connection_request_rx,
        connection_accepted_tx: config.connection_accepted_tx,
    });
}

async fn handle_datagram(
    state: &mut State,
    dgram: DatagramRecv,
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
                        ).await;

                        state.connection_accepted_tx.send(connection).unwrap(); // TODO: Handle error
                    },

                    Err(_) => todo!(),
                }
            },

            quinn_proto::DatagramEvent::Response(transmit) => {
                let mut payload = BytesMut::with_capacity(transmit.size);
                payload.copy_from_slice(&scratch[..transmit.size]);

                state.socket_dgram_send_tx.send(DatagramSend {
                    target: transmit.destination,
                    payload,
                }).unwrap(); // TODO: Handle error
            },
        }
    }
}

async fn handle_event(
    state: &mut State,
    event: EndpointEvent,
) {
    if let Some(response) = state.quinn.handle_event(event.id, event.data) {
        let handle = state.connections.get(&event.id).unwrap();
        handle.quinn_event_tx.send(response).unwrap(); // TODO: Handle error
    }
}

async fn handle_connection_request(
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
            ).await);
        },

        Err(err) => {
            request.inner.reject(ConnectionError::QuicError(err));
        },
    }
}

async fn add_connection(
    state: &mut State,
    id: QuinnConnectionId,
    quinn: quinn_proto::Connection,
) -> NewConnection {
    let (quinn_event_tx, quinn_event_rx) = mpsc::unbounded_channel();

    state.connections.insert(id, ConnectionHandle {
        quinn_event_tx,
    });

    return NewConnection {
        quinn,

        endpoint: EndpointHandle {
            quinn_event_rx,
            quinn_event_tx: state.quinn_event_tx.clone(),
        },
    };
}

async fn io_recv_task(
    socket: Arc<UdpSocket>,
    socket_dgram_recv_tx: mpsc::UnboundedSender<DatagramRecv>,
) {
    loop {
        let mut payload = BytesMut::with_capacity(2048); // TODO: Increase this size

        match socket.recv_buf_from(&mut payload).await {
            Ok((_, origin)) => {
                let message = DatagramRecv {
                    origin,
                    payload,
                };

                if let Err(_) = socket_dgram_recv_tx.send(message) {
                    return; // Channel is closed
                }
            },

            Err(e) if e.kind() == ErrorKind::WouldBlock => {},

            Err(_) => todo!(),
        }
    }
}

async fn io_send_task(
    socket: Arc<UdpSocket>,
    mut socket_dgram_send_rx: mpsc::UnboundedReceiver<DatagramSend>,
) {
    while let Some(dgram) = socket_dgram_send_rx.recv().await {
        match socket.send_to(&dgram.payload, dgram.target).await {
            Ok(_) => continue, // Success

            Err(e) if e.kind() == ErrorKind::WouldBlock => todo!(),

            Err(_) => todo!(),
        }
    }
}