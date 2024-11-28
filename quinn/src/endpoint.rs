use std::{collections::HashMap, sync::Arc};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnConnectionId, EndpointEvent};
use tokio::{net::UdpSocket, sync::{mpsc, Mutex, Notify}, task::JoinHandle};
use crate::{commands::MakeEndpointInner, connection::ConnectionRef};

pub struct Endpoint {
    inner: EndpointRef,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

#[derive(Clone)]
pub(crate) struct EndpointRef(Arc<EndpointInner>);

struct EndpointInner {
    state: Mutex<State>,
    shared: Shared,
}

enum State {
    Building(Building),
    Established(Established),
}

struct Building(JoinHandle<Result<Established, BuildError>>);

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

struct Established {
    socket: Arc<UdpSocket>,

    quinn: quinn_proto::Endpoint,

    connections: HashMap<QuinnConnectionId, ConnectionHandle>,
}

struct Shared {
    runtime: tokio::runtime::Handle,
    wakeup: Notify,
}

struct ConnectionHandle {
    inner_ref: ConnectionRef,

    event_tx: mpsc::UnboundedSender<ConnectionEvent>,
    event_rx: mpsc::UnboundedReceiver<EndpointEvent>,
}

fn build(
    runtime: tokio::runtime::Handle,
    config: MakeEndpointInner,
) -> (
    Shared,
    Building,
) {
    // Create shared state object
    let shared = Shared {
        runtime: runtime.clone(),
        wakeup: Notify::new(),
    };

    // Start the building task so it executes in the background
    let building = Building(runtime.spawn(build_task(
        runtime.clone(),
        config,
    )));

    return (shared, building);
}

async fn build_task(
    runtime: tokio::runtime::Handle,
    config: MakeEndpointInner
) -> Result<Established, BuildError> {
    let (socket, quinn) = match config {
        MakeEndpointInner::Preconfigured {
            socket,
            config,
            server,
        } => {
            // Configure UDP socket
            socket.set_nonblocking(true)?;

            // Create Tokio socket
            let socket = UdpSocket::from_std(socket)?;

            // Create Quinn endpoint
            let quinn = quinn_proto::Endpoint::new(
                config,
                server,
                true,
                None,
            );

            (socket, quinn)
        },
    };

    return Ok(Established {
        socket: Arc::new(socket),
        quinn,
        connections: HashMap::new(),
    });
}