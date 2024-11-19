use std::{collections::HashMap, io::ErrorKind, net::SocketAddr, sync::Arc, time::Instant};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use bytes::BytesMut;
use quinn_proto::DatagramEvent;
use tokio::sync::oneshot::error::TryRecvError;
use crate::commands::MakeEndpointInner;

pub struct Endpoint {
    handle: tokio::task::JoinHandle<()>,
    state: tokio::sync::watch::Receiver<EndpointState>,
    close: Option<tokio::sync::oneshot::Sender<()>>,
    wakeup: Arc<tokio::sync::Notify>,
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
        self.state.borrow().clone()
    }

    pub fn close(
        &mut self,
    ) {
        // If the event is run already, don't bother
        if self.close.is_none() { return }

        // Send the closer one-shot event
        let mut closer = None;
        std::mem::swap(&mut closer, &mut self.close);
        let closer = closer.unwrap();
        let _ = closer.send(());
    }
}

struct State {
    runtime: tokio::runtime::Handle,
    closer: tokio::sync::oneshot::Receiver<()>,
    state: tokio::sync::watch::Sender<EndpointState>,
    wakeup: Arc<tokio::sync::Notify>,

    connections: HashMap<
        quinn_proto::ConnectionHandle,
        Connection,
    >,

    socket: Arc<tokio::net::UdpSocket>,

    socket_dgrams_recv_rx: tokio::sync::mpsc::UnboundedReceiver<DatagramRecv>,
    socket_dgrams_send_tx: tokio::sync::mpsc::UnboundedSender<DatagramSend>,

    quinn: quinn_proto::Endpoint,

    quinn_events_rx: tokio::sync::mpsc::UnboundedReceiver<(
        quinn_proto::ConnectionHandle,
        quinn_proto::EndpointEvent,
    )>,
}

struct Connection {
    wakeup: Arc<tokio::sync::Notify>,

    quinn_events_tx: tokio::sync::mpsc::UnboundedSender<
        quinn_proto::ConnectionEvent,
    >,
}

pub(crate) fn open(
    runtime: tokio::runtime::Handle,
    make_endpoint: MakeEndpointInner,
) -> Endpoint {
    let (state_tx, state_rx) = tokio::sync::watch::channel(EndpointState::Building);
    let (closer_tx, closer_rx) = tokio::sync::oneshot::channel();
    let wakeup = Arc::new(tokio::sync::Notify::new());

    return Endpoint {
        handle: runtime.spawn(build(
            runtime.clone(),
            BuildMeta {
                state: state_tx,
                closer: closer_rx,
            },
            make_endpoint,
        )),

        state: state_rx,
        close: Some(closer_tx),
        wakeup,
    }
}

struct BuildMeta {
    state: tokio::sync::watch::Sender<EndpointState>,
    closer: tokio::sync::oneshot::Receiver<()>,
}

async fn build(
    runtime: tokio::runtime::Handle,
    meta: BuildMeta,
    make_endpoint: MakeEndpointInner,
) {

}

struct RunMeta {
    socket_dgrams_recv_tx: tokio::sync::mpsc::UnboundedSender<DatagramRecv>,
    socket_dgrams_send_rx: tokio::sync::mpsc::UnboundedReceiver<DatagramSend>,
}

async fn run(
    mut state: State,
    meta: RunMeta,
) {
    // Spawn I/O receiving task
    state.runtime.spawn(io_recv(
        state.socket.clone(),
        meta.socket_dgrams_recv_tx,
        state.wakeup.clone()
    ));

    // Spawn I/O sending task
    state.runtime.spawn(io_send(
        state.socket.clone(),
        meta.socket_dgrams_send_rx,
    ));

    loop {
        if let Some(end) = tick(&mut state).await {
            break; // Break out
        }

        state.wakeup.notified().await;
    }
}

async fn tick(
    state: &mut State,
) -> Option<()> {
    // Iterate over incoming connection events
    while let Ok((handle, event)) = state.quinn_events_rx.try_recv() {
        if let Some(event) = state.quinn.handle_event(handle, event) {
            let connection = state.connections.get(&handle).unwrap(); // TODO: Handle error
            connection.quinn_events_tx.send(event).unwrap(); // TODO: Handle error
            connection.wakeup.notify_waiters();
        }
    }

    // Iterate over received datagrams
    let mut scratch = Vec::new();
    while let Ok(recv) = state.socket_dgrams_recv_rx.try_recv() {
        match state.quinn.handle(
            Instant::now(),
            recv.origin,
            None,
            None,
            recv.data,
            &mut scratch,
        ) {
            Some(DatagramEvent::ConnectionEvent(handle, event)) => {
                let connection = state.connections.get(&handle).unwrap(); // TODO: Handle error
                connection.quinn_events_tx.send(event).unwrap(); // TODO: Handle error
                connection.wakeup.notify_waiters();
            },

            Some(DatagramEvent::NewConnection(incoming)) => {
                match state.quinn.accept(
                    incoming,
                    Instant::now(),
                    &mut scratch,
                    None,
                ) {
                    Ok(_) => todo!(),
                    Err(_) => todo!(),
                }
            }

            Some(DatagramEvent::Response(transmit)) => {
                let mut buf = BytesMut::with_capacity(scratch.len());
                buf.extend_from_slice(&scratch);

                state.socket_dgrams_send_tx.send(DatagramSend {
                    data: buf,
                    target: transmit.destination,
                }).unwrap(); // TODO: Handle error
            }

            None => { /* Do nothing */ },
        }
    }

    // See if the closer has been fired
    match state.closer.try_recv() {
        Ok(()) => return Some(()),
        Err(TryRecvError::Empty) => return None,
        Err(TryRecvError::Closed) => return Some(()),
    }
}

struct DatagramRecv {
    data: BytesMut,
    origin: SocketAddr,
}

async fn io_recv(
    socket: Arc<tokio::net::UdpSocket>,
    socket_dgrams_recv_tx: tokio::sync::mpsc::UnboundedSender<DatagramRecv>,
    wakeup: Arc<tokio::sync::Notify>,
) {
    loop {
        // Wait for the socket to become readable
        socket.readable().await.unwrap(); // TODO: Handle error

        let mut buf = BytesMut::with_capacity(1024);
        match socket.try_recv_buf_from(&mut buf) {
            Ok((_len, origin)) => {
                // Push datagram to queue for endpoint
                socket_dgrams_recv_tx.send(DatagramRecv {
                    data: buf,
                    origin,
                }).unwrap(); // TODO: Handle error

                // Wake up notifier task
                wakeup.notify_waiters();
            },

            // Shouldn't happen
            Err(e) if e.kind() == ErrorKind::WouldBlock => unimplemented!(),

            Err(e) => todo!(),
        }
    }
}

struct DatagramSend {
    data: BytesMut,
    target: SocketAddr,
}

async fn io_send(
    socket: Arc<tokio::net::UdpSocket>,
    mut socket_dgrams_send_rx: tokio::sync::mpsc::UnboundedReceiver<DatagramSend>,
) {
    loop {
        // Wait for a datagram
        let dgram = socket_dgrams_send_rx.recv().await.unwrap(); // TODO: Handle error

        // Send datagram
        match socket.send_to(&dgram.data, dgram.target).await {
            Ok(_) => todo!(),

            // Shouldn't happen
            Err(e) if e.kind() == ErrorKind::WouldBlock => unimplemented!(),

            Err(e) => todo!(),
        }
    }
}