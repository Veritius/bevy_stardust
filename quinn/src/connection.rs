use std::{future::Future, net::SocketAddr, sync::Arc, task::Poll};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use bevy_stardust::prelude::ChannelMessage;
use futures_lite::FutureExt;
use tokio::{select, sync::{mpsc, watch}, task::JoinHandle};
use crate::endpoint::EndpointHandle;

pub struct Connection {
    pub(crate) handle: Handle,

    driver: JoinHandle<()>,
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

struct State {
    state: watch::Sender<ConnectionState>,

    quinn: quinn_proto::Connection,

    outgoing_messages_rx: mpsc::Receiver<ChannelMessage>,
    incoming_messages_tx: mpsc::Sender<ChannelMessage>,
}

pub(crate) struct Handle {
    state: watch::Receiver<ConnectionState>,

    outgoing_messages_tx: mpsc::Sender<ChannelMessage>,
    incoming_messages_rx: mpsc::Receiver<ChannelMessage>,
}

pub enum ConnectionState {
    Connecting,
    Connected,
    Shutdown
}

pub(crate) struct ConnectionRequest {
    pub data: ConnectionRequestData,
    pub inner: ConnectionRequestInner,
}

pub(crate) struct ConnectionRequestData {
    pub client_config: quinn_proto::ClientConfig,
    pub address: SocketAddr,
    pub server_name: Arc<str>,
}

pub(crate) struct ConnectionRequestInner {
    request_tx: tokio::sync::oneshot::Sender<
        Result<NewConnection, ConnectionError>,
    >,
}

impl ConnectionRequestInner {
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
    pub endpoint: EndpointHandle,
}

pub(crate) enum ConnectionError {
    EndpointClosed,
    QuicError(quinn_proto::ConnectError),
}

async fn tick(
    state: &mut State,
) {
    select! {
        message = state.outgoing_messages_rx.recv() => match message {
            Some(message) => handle_outgoing_message(state, message).await,
            None => todo!(),
        },
    }
}

async fn handle_outgoing_message(
    state: &mut State,
    message: ChannelMessage,
) {

}