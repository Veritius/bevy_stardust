use std::{future::Future, net::SocketAddr, sync::Arc, task::Poll};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use bevy_stardust::prelude::ChannelMessage;
use futures_lite::FutureExt;
use quinn_proto::ConnectionEvent;
use tokio::{select, sync::{mpsc, oneshot, watch}, task::JoinHandle, runtime::Handle as RuntimeHandle};
use crate::{endpoint::EndpointHandle, Endpoint, Runtime};

/// A handle to an existing connection.
/// 
/// This component can be transferred freely.
/// When dropped, the connection will be closed.
pub struct Connection {
    pub(crate) handle: Handle,

    driver: JoinHandle<()>,
}

impl Connection {
    /// Creates a connection to a remote target.
    pub fn connect(
        runtime: &Runtime,
        endpoint: &Endpoint,
        config: quinn_proto::ClientConfig,
        address: SocketAddr,
        server_name: Arc<str>,
    ) -> Result<Connection, ConnectionError> {
        let (request_tx, request_rx) = tokio::sync::oneshot::channel();

        endpoint.handle.connect(ConnectionRequest {
            data: ConnectionRequestData {
                client_config: config,
                address,
                server_name,
            },
            inner: ConnectionRequestInner { request_tx },
        })?;

        let (handle, driver) = outgoing(
            runtime.handle(),
            ConnectionRequestResponseListener { request_rx },
        );

        return Ok(Connection {
            handle,
            driver,
        });
    }
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

struct State {
    state: watch::Sender<ConnectionState>,
    shutdown: oneshot::Receiver<()>,

    endpoint: EndpointHandle,

    quinn: quinn_proto::Connection,

    outgoing_messages_rx: mpsc::UnboundedReceiver<ChannelMessage>,
    incoming_messages_tx: mpsc::UnboundedSender<ChannelMessage>,
}

pub(crate) struct Handle {
    state_rx: watch::Receiver<ConnectionState>,
    shutdown_tx: Option<oneshot::Sender<()>>,

    outgoing_messages_tx: mpsc::UnboundedSender<ChannelMessage>,
    incoming_messages_rx: mpsc::UnboundedReceiver<ChannelMessage>,
}

/// The state of the connection.
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

/// An error produced by a [`Connection`].
#[derive(Debug)]
pub enum ConnectionError {
    EndpointClosed,
    QuicError(quinn_proto::ConnectError),
}

struct BuildData {
    state_tx: watch::Sender<ConnectionState>,
    shutdown_rx: oneshot::Receiver<()>,

    outgoing_messages_rx: mpsc::UnboundedReceiver<ChannelMessage>,
    incoming_messages_tx: mpsc::UnboundedSender<ChannelMessage>,
}

fn outgoing(
    runtime: RuntimeHandle,
    listener: ConnectionRequestResponseListener,
) -> (Handle, JoinHandle<()>) {
    let (state_tx, state_rx) = watch::channel(ConnectionState::Connecting);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (outgoing_messages_tx, outgoing_messages_rx) = mpsc::unbounded_channel();
    let (incoming_messages_tx, incoming_messages_rx) = mpsc::unbounded_channel();

    // Spawn task
    let task = runtime.spawn(build(
        runtime.clone(),
        listener,
        BuildData {
            state_tx,
            shutdown_rx,
            outgoing_messages_rx,
            incoming_messages_tx,
        },
    ));

    // Create handle
    let handle =  Handle {
        state_rx,
        shutdown_tx: Some(shutdown_tx),
        outgoing_messages_tx,
        incoming_messages_rx,
    };

    return (handle, task);
}

async fn build(
    runtime: RuntimeHandle,
    listener: ConnectionRequestResponseListener,
    data: BuildData,
) {
    let connection = match listener.await {
        Ok(c) => c,
        Err(_) => todo!(),
    };

    task(
        runtime,
        connection,
        data
    ).await
}

async fn task(
    runtime: RuntimeHandle,
    connection: NewConnection,
    data: BuildData,
) {
    let state = State {
        state: data.state_tx,
        shutdown: data.shutdown_rx,
        endpoint: connection.endpoint,
        quinn: connection.quinn,
        outgoing_messages_rx: data.outgoing_messages_rx,
        incoming_messages_tx: data.incoming_messages_tx,
    };
}

async fn tick(
    state: &mut State,
) {
    select! {
        event = state.endpoint.recv_connection_event() => match event {
            Some(event) => handle_connection_event(state, event).await,
            None => todo!(),
        },

        message = state.outgoing_messages_rx.recv() => match message {
            Some(message) => handle_outgoing_message(state, message).await,
            None => todo!(),
        },
    }
}

async fn handle_connection_event(
    state: &mut State,
    event: ConnectionEvent,
) {
    state.quinn.handle_event(event);
}

async fn handle_outgoing_message(
    state: &mut State,
    message: ChannelMessage,
) {

}