use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use bytes::BytesMut;
use tokio::{net::UdpSocket, sync::mpsc::{UnboundedReceiver, UnboundedSender}};
use crate::commands::MakeEndpointInner;

pub struct Endpoint {
    task: tokio::task::JoinHandle<()>,
    inner: Arc<EndpointInner>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndpointState {
    Building,
    Established,
    Closed,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            let mut entity = world.entity_mut(entity);
            let mut component = entity.get_mut::<Endpoint>().unwrap();
            component.close();
        });
    }
}

impl Endpoint {
    pub fn state(&self) -> EndpointState {
        self.inner.state_rx.borrow().clone()
    }

    pub fn close(
        &mut self,
    ) {
        todo!()
    }
}

struct EndpointInner {
    runtime: tokio::runtime::Handle,
    notify: tokio::sync::Notify,

    inner: tokio::sync::Mutex<State>,
    state_rx: tokio::sync::watch::Receiver<EndpointState>,

    quinn_events_rx: tokio::sync::mpsc::UnboundedReceiver<(
        quinn_proto::ConnectionHandle,
        quinn_proto::EndpointEvent,
    )>,
}

impl EndpointInner {
    fn state(&self) -> EndpointState {
        self.state_rx.borrow().clone()
    }
}

enum State {
    Building,
    Established(Established),
}

struct Established {
    socket: Arc<UdpSocket>,

    socket_dgram_recv_rx: UnboundedReceiver<DatagramRecv>,
    socket_dgram_send_tx: UnboundedSender<DatagramSend>,
    socket_recv_task: tokio::task::JoinHandle<()>,
    socket_send_task: tokio::task::JoinHandle<()>,

    connections: HashMap<
        quinn_proto::ConnectionHandle,
        ConnectionHandle,
    >,

    quinn: quinn_proto::Endpoint,

    quinn_events_tx: tokio::sync::mpsc::UnboundedSender<(
        quinn_proto::ConnectionHandle,
        quinn_proto::EndpointEvent,
    )>,
}

struct ConnectionHandle {
    quinn_events_tx: tokio::sync::mpsc::UnboundedSender<
        quinn_proto::ConnectionEvent,
    >,
}

struct DatagramRecv {
    data: BytesMut,
    origin: SocketAddr,
}

struct DatagramSend {
    data: BytesMut,
    target: SocketAddr,
}

pub(crate) fn open(
    runtime: tokio::runtime::Handle,
    make_endpoint: MakeEndpointInner,
) -> Endpoint {
    let (state_tx, state_rx) = tokio::sync::watch::channel(EndpointState::Building);
    let (quinn_events_tx, quinn_events_rx) = tokio::sync::mpsc::unbounded_channel();

    let inner = Arc::new(EndpointInner {
        runtime: runtime.clone(),
        notify: tokio::sync::Notify::new(),

        inner: tokio::sync::Mutex::new(State::Building),

        state_rx,

        quinn_events_rx,
    });

    return Endpoint {
        task: runtime.clone().spawn(run(
            runtime,
            inner.clone(),
            make_endpoint,

            RunMeta {
                state_tx,
                quinn_events_tx,
            },
        )),

        inner,
    };
}

struct RunMeta {
    state_tx: tokio::sync::watch::Sender<EndpointState>,
    quinn_events_tx: tokio::sync::mpsc::UnboundedSender<(
        quinn_proto::ConnectionHandle,
        quinn_proto::EndpointEvent,
    )>,
}

async fn run(
    runtime: tokio::runtime::Handle,
    inner: Arc<EndpointInner>,
    make_endpoint: MakeEndpointInner,
    meta: RunMeta,
) {
    let RunMeta {
        state_tx,
        quinn_events_tx,
    } = meta;

    let (socket, quinn) = match make_endpoint {
        MakeEndpointInner::Preconfigured {
            socket,
            config,
            server,
        } => {
            // TODO: Handle error
            let socket = Arc::new(
                UdpSocket::from_std(socket).unwrap()
            );

            let quinn = quinn_proto::Endpoint::new(
                config,
                server,
                true,
                None,
            );

            (socket, quinn)
        },
    };
    // Construct additional channels
    let (socket_dgram_send_tx, socket_dgram_send_rx) = tokio::sync::mpsc::unbounded_channel();
    let (socket_dgram_recv_tx, socket_dgram_recv_rx) = tokio::sync::mpsc::unbounded_channel();

    // Construct the established state object
    let est = Established {
        socket: socket.clone(),

        socket_dgram_recv_rx,
        socket_dgram_send_tx,

        socket_recv_task: runtime.spawn(recv(
            socket.clone(),
            socket_dgram_recv_tx,
        )),

        socket_send_task: runtime.spawn(send(
            socket.clone(),
            socket_dgram_send_rx,
        )),

        connections: HashMap::new(),

        quinn,
        quinn_events_tx,
    };

    // Set the endpoint to Established as it is successful
    let mut lock = inner.inner.lock().await;
    *lock = State::Established(est);
    drop(lock);

    // Notify other tasks that the endpoint is now established
    state_tx.send(EndpointState::Established).unwrap();

    loop {
        // Tick, handling updates
        tick(runtime.clone(), &inner).await;

        // Wait for a new notification
        inner.notify.notified().await;
    }
}

async fn tick(
    runtime: tokio::runtime::Handle,
    inner: &EndpointInner,
) {
    todo!()
}

async fn recv(
    socket: Arc<UdpSocket>,
    socket_dgram_recv_tx: UnboundedSender<DatagramRecv>,
) {

}

async fn send(
    socket: Arc<UdpSocket>,
    socket_dgram_send_rx: UnboundedReceiver<DatagramSend>,
) {

}