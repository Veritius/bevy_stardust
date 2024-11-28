use std::{future::Future, sync::Arc, task::Poll};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use futures_lite::FutureExt;
use quinn_proto::ConnectionEvent;
use tokio::sync::{mpsc, Mutex, Notify};

use crate::endpoint::EndpointEvent;

pub struct Connection {
    inner: ConnectionRef,
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

#[derive(Clone)]
pub(crate) struct ConnectionRef(Arc<EndpointInner>);

struct EndpointInner {
    state: Mutex<State>,
    shared: Shared,
}

struct State {
    quinn: quinn_proto::Connection,
}

struct Shared {
    runtime: tokio::runtime::Handle,
    wakeup: Notify,

    endpoint: EndpointHandle,
}

struct EndpointHandle {
    quinn_event_tx: mpsc::UnboundedSender<EndpointEvent>,
    quinn_event_rx: mpsc::UnboundedReceiver<ConnectionEvent>,
}

pub(crate) struct ConnectionRequest {
    request_tx: tokio::sync::oneshot::Sender<
        Result<NewConnection, ConnectionError>,
    >,
}

impl ConnectionRequest {
    fn new() -> (
        ConnectionRequest,
        ConnectionRequestResponseListener,
    ) {
        let (request_tx, request_rx) = tokio::sync::oneshot::channel();

        let tx = ConnectionRequest { request_tx };
        let rx = ConnectionRequestResponseListener { request_rx };

        return (tx, rx);
    }
}

impl ConnectionRequest {
    pub fn accept(self, connection: NewConnection) {
        let _ = self.request_tx.send(Ok(connection));
    }

    pub fn reject(self, error: ConnectionError) {
        let _ = self.request_tx.send(Err(error));
    }
}

struct ConnectionRequestResponseListener {
    request_rx: tokio::sync::oneshot::Receiver<
        Result<NewConnection, ConnectionError>
    >,
}

impl Future for ConnectionRequestResponseListener {
    type Output = Result<NewConnection, ConnectionError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        match self.request_rx.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(v)) => Poll::Ready(v),
            Poll::Ready(Err(_)) => Poll::Ready(Err(ConnectionError::EndpointClosed)),
        }
    }
}

pub(crate) struct NewConnection {
    pub quinn: quinn_proto::Connection,

    pub quinn_event_tx: mpsc::UnboundedSender<EndpointEvent>,
    pub quinn_event_rx: mpsc::UnboundedReceiver<ConnectionEvent>,
}

pub(crate) enum ConnectionError {
    EndpointClosed,
}