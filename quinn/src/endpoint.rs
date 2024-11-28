use std::{collections::HashMap, sync::Arc, time::Duration};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnConnectionId};
use tokio::{net::UdpSocket, sync::{mpsc, Mutex, Notify}, task::JoinHandle};
use crate::{commands::MakeEndpointInner, connection::ConnectionRef};

pub struct Endpoint {
    driver: JoinHandle<()>,
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

    quinn_event_rx: mpsc::UnboundedReceiver<EndpointEvent>,
    // Local copy of the sender for quinn_event_rx so it can be given to connections
    quinn_event_tx: mpsc::UnboundedSender<EndpointEvent>,
}

pub(crate) struct EndpointEvent {
    pub id: quinn_proto::ConnectionHandle,
    pub evt: quinn_proto::EndpointEvent,
}

struct ConnectionHandle {
    inner_ref: ConnectionRef,

    quinn_event_tx: mpsc::UnboundedSender<ConnectionEvent>,
}

fn build(
    runtime: tokio::runtime::Handle,
    config: MakeEndpointInner,
) -> (
    Shared,
    Building,
) {
    // Create various communication channels
    let (quinn_event_tx, quinn_event_rx) = mpsc::unbounded_channel();

    // Create shared state object
    let shared = Shared {
        runtime: runtime.clone(),
        wakeup: Notify::new(),

        quinn_event_rx,
        quinn_event_tx,
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

async fn driver(
    runtime: tokio::runtime::Handle,
    endpoint: EndpointRef,
) {
    loop {
        // Tick the endpoint once
        let tick = tick(
            &runtime,
            &endpoint.0,
        ).await;

        // Wait for the next notification from another part of the code
        // Can also return early after a time without notifications
        // This is controlled by [`tick`] and handles internal timers
        let _ = tokio::time::timeout(
            tick.timeout,
            endpoint.0.shared.wakeup.notified(),
        ).await;
    }
}

async fn tick(
    runtime: &tokio::runtime::Handle,
    endpoint: &EndpointInner,
) -> TickOutput {
    todo!()
}

struct TickOutput {
    timeout: Duration,
}