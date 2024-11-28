use std::{collections::HashMap, sync::Arc, time::Duration};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnConnectionId};
use tokio::{net::UdpSocket, runtime::Handle as RuntimeHandle, sync::{mpsc, Mutex, Notify}, task::JoinHandle};
use crate::{commands::MakeEndpointInner, connection::{ConnectionError, ConnectionRef, ConnectionRequest, NewConnection}};

pub struct Endpoint {
    pub(crate) shared: EndpointShared,

    driver: JoinHandle<()>,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

struct EndpointState {
    waker: Arc<Notify>,

    socket: Arc<UdpSocket>,

    quinn: quinn_proto::Endpoint,

    quinn_event_rx: mpsc::UnboundedReceiver<EndpointEvent>,
    quinn_event_tx: mpsc::UnboundedSender<EndpointEvent>,

    connections: HashMap<QuinnConnectionId, ConnectionHandle>,

    connection_request_rx: mpsc::UnboundedReceiver<ConnectionRequest>,
}

pub(crate) struct EndpointShared {
    waker: Arc<Notify>,

    connection_request_tx: mpsc::UnboundedSender<ConnectionRequest>,
}

impl EndpointShared {
    pub fn connect(&self, request: ConnectionRequest) -> Result<(), ConnectionError> {
        self.connection_request_tx.send(request).map_err(|_| ConnectionError::EndpointClosed)?;
        self.waker.notify_one();
        return Ok(());
    }
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
    todo!()
}

async fn endpoint(

) {

}

async fn build(

) -> Result<EndpointState, BuildError> {
    todo!()
}

async fn driver(

) {

}