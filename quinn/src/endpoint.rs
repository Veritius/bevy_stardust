use std::{collections::HashMap, io::ErrorKind, net::SocketAddr, os::linux::raw::stat, sync::{Arc, Mutex}, time::Instant};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use bytes::BytesMut;
use quinn_proto::DatagramEvent;
use tokio::sync::oneshot::error::TryRecvError;
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

    inner: Mutex<State>,
    state_rx: tokio::sync::watch::Receiver<EndpointState>,

    quinn_events_rx: tokio::sync::mpsc::UnboundedReceiver<(
        quinn_proto::ConnectionHandle,
        quinn_proto::EndpointEvent,
    )>,

    socket: tokio::net::UdpSocket,

    socket_dgrams_recv_rx: tokio::sync::mpsc::UnboundedReceiver<DatagramRecv>,
    socket_dgrams_send_tx: tokio::sync::mpsc::UnboundedSender<DatagramSend>,
}

enum State {
    Building,
    Established(Established),
}

struct Established {
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
    let (socket_dgrams_recv_tx, socket_dgrams_recv_rx) = tokio::sync::mpsc::unbounded_channel();
    let (socket_dgrams_send_tx, socket_dgrams_send_rx) = tokio::sync::mpsc::unbounded_channel();
    let (quinn_events_tx, quinn_events_rx) = tokio::sync::mpsc::unbounded_channel();

    let inner = Arc::new(EndpointInner {
        runtime,

        inner: Mutex::new(State::Building),

        state_rx,

        quinn_events_rx,

        socket: todo!(),
        socket_dgrams_recv_rx,
        socket_dgrams_send_tx,
    });

    return Endpoint {
        task: runtime.spawn(run(
            runtime,
            inner,
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

    // Construct the Quinn endpoint state object
    let quinn = quinn_proto::Endpoint::new(
        todo!(),
        todo!(),
        true,
        None,
    );

    // Construct the established state object
    let est = Established {
        connections: HashMap::new(),

        quinn,
        quinn_events_tx,
    };

    // Set the endpoint to Established as it is successful
    let mut lock = inner.inner.lock().unwrap();
    *lock = State::Established(est);
    drop(lock);

    // Notify other tasks that the endpoint is now established
    state_tx.send(EndpointState::Established).unwrap();
}