use std::{collections::HashMap, sync::Arc};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnConnectionId};
use tokio::{net::UdpSocket, runtime::Handle as RuntimeHandle, sync::{mpsc, watch, Notify}, task::JoinHandle};
use crate::{commands::MakeEndpointInner, connection::{ConnectionError, ConnectionRef, ConnectionRequest}};

pub struct Endpoint {
    pub(crate) handle: Handle,

    driver: JoinHandle<()>,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

struct State {
    waker: Arc<Notify>,
    state: watch::Sender<EndpointState>,

    socket: Arc<UdpSocket>,

    quinn: quinn_proto::Endpoint,

    quinn_event_rx: mpsc::UnboundedReceiver<EndpointEvent>,
    quinn_event_tx: mpsc::UnboundedSender<EndpointEvent>,

    connections: HashMap<QuinnConnectionId, ConnectionHandle>,

    connection_request_rx: mpsc::UnboundedReceiver<ConnectionRequest>,
}

pub(crate) struct Handle {
    waker: Arc<Notify>,
    state: watch::Receiver<EndpointState>,

    connection_request_tx: mpsc::UnboundedSender<ConnectionRequest>,
}

impl Handle {
    pub fn connect(&self, request: ConnectionRequest) -> Result<(), ConnectionError> {
        self.connection_request_tx.send(request).map_err(|_| ConnectionError::EndpointClosed)?;
        self.waker.notify_one();
        return Ok(());
    }
}

pub enum EndpointState {
    Building,
    Established,
    Shutdown,
}

pub(crate) struct EndpointEvent {
    pub id: quinn_proto::ConnectionHandle,
    pub evt: quinn_proto::EndpointEvent,
}

struct ConnectionHandle {
    inner_ref: ConnectionRef,

    quinn_event_tx: mpsc::UnboundedSender<ConnectionEvent>,
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

fn open(
    runtime: RuntimeHandle,
    config: MakeEndpointInner,
) -> Endpoint {
    // Endpoint waker and state storage
    let waker = Arc::new(Notify::new());

    // Create various communication channels
    let (state_tx, state_rx) = tokio::sync::watch::channel(EndpointState::Building);
    let (connection_request_tx, connection_request_rx) = mpsc::unbounded_channel();

    Endpoint {
        handle: Handle {
            waker,
            state: state_rx,

            connection_request_tx,
        },

        driver: runtime.spawn(endpoint(BuildTaskData {
            config,

            state_tx,
            connection_request_rx,
        })),
    }
}

async fn endpoint(
    config: BuildTaskData,
) {

}

struct BuildTaskData {
    config: MakeEndpointInner,

    state_tx: watch::Sender<EndpointState>,
    connection_request_rx: mpsc::UnboundedReceiver<ConnectionRequest>,
}

async fn build(
    config: BuildTaskData,
) -> Result<State, BuildError> {
    todo!()
}

async fn driver(

) {

}