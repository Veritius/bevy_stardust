use std::{collections::HashMap, future::Future, net::{SocketAddr, ToSocketAddrs, UdpSocket}, pin::{pin, Pin}, sync::Arc};
use async_channel::{Receiver, Sender};
use async_io::Async;
use async_task::Task;
use quinn_proto::{crypto::ServerConfig, ConnectionHandle, EndpointConfig, EndpointEvent, TransportConfig};
use crate::{events::{C2EEvent, E2CEvent}, futures::Race, taskpool::{get_task_pool, NetworkTaskPool}};

/// A builder for an [`Endpoint`].
pub struct EndpointBuilder<S = ()> {
    task_pool: &'static NetworkTaskPool,
    state: S,
}

impl EndpointBuilder<()> {
    /// Creates a new [`EndpointBuilder`].
    pub fn new() -> EndpointBuilder::<WantsSocket> {
        EndpointBuilder {
            task_pool: get_task_pool(),
            state: WantsSocket { _p: () },
        }
    }
}

/// State for adding a socket.
pub struct WantsSocket {
    _p: (),
}

impl EndpointBuilder<WantsSocket> {
    /// Uses a pre-existing standard library UDP socket.
    pub fn use_existing(self, socket: UdpSocket) -> EndpointBuilder<WantsQuicConfig> {
        let socket = blocking::unblock(move || Async::new(socket));

        EndpointBuilder {
            task_pool: self.task_pool,
            state: WantsQuicConfig { socket },
        }
    }

    /// Binds to the given address, creating a new socket.
    pub fn bind<A>(self, address: A) -> EndpointBuilder<WantsQuicConfig>
    where
        A: ToSocketAddrs,
        A: Send + Sync + 'static,
        A::Iter: Send + Sync + 'static,
    {
        // We have to bind the socket manually with blocking because AsyncToSocketAddrs has weird trait requirements
        // that can never be fulfilled while trying to use simple async closures like this. Oh well, it's good enough.
        let socket = blocking::unblock(move || Async::new(UdpSocket::bind(address)?));

        EndpointBuilder {
            task_pool: self.task_pool,
            state: WantsQuicConfig { socket },
        }
    }
}

/// State for adding a reset key.
pub struct WantsQuicConfig {
    socket: Task<Result<Async<UdpSocket>, std::io::Error>>,
}

impl EndpointBuilder<WantsQuicConfig> {
    /// Uses an existing reset key.
    pub fn use_existing(self, config: Arc<EndpointConfig>) -> EndpointBuilder<CanBecomeServer> {
        EndpointBuilder {
            task_pool: self.task_pool,
            state: CanBecomeServer {
                previous: self.state,
                config,
            },
        }
    }
}

/// State for optionally configuring server behavior.
pub struct CanBecomeServer {
    previous: WantsQuicConfig,
    config: Arc<EndpointConfig>,
}

impl EndpointBuilder<CanBecomeServer> {
    /// Skips server configuration.
    pub fn client_only(self) -> LoadingEndpoint {
        LoadingEndpoint(self.task_pool.spawn(async move {
            Endpoint::new_inner(
                self.state.previous.socket,
                self.state.config,
                async { None },
            ).await
        }))
    }
}

/// State for setting a [`TransportConfig`] value.
pub struct WantsTransportConfig {
    previous: CanBecomeServer,
}

impl EndpointBuilder<WantsTransportConfig> {
    /// Uses the default transport configuration suitable for most applications.
    pub fn use_default(self) -> EndpointBuilder<WantsServerCrypto> {
        EndpointBuilder {
            task_pool: self.task_pool,
            state: WantsServerCrypto {
                previous: self.state,
                config: Arc::new(TransportConfig::default()),
            },
        }
    }

    /// Uses an existing transport configuration value.
    pub fn use_existing(self, transport_config: Arc<TransportConfig>) -> EndpointBuilder<WantsServerCrypto> {
        EndpointBuilder {
            task_pool: self.task_pool,
            state: WantsServerCrypto {
                previous: self.state,
                config: transport_config,
            },
        }
    }
}

/// State for adding cryptographic data.
pub struct WantsServerCrypto {
    previous: WantsTransportConfig,
    config: Arc<TransportConfig>,
}

impl EndpointBuilder<WantsServerCrypto> {
    /// Uses an existing server configuration value.
    pub fn use_existing(
        self,
        server_config: Arc<dyn ServerConfig>
    ) -> LoadingEndpoint {
        LoadingEndpoint(self.task_pool.spawn(async move {
            Endpoint::new_inner(
                self.state.previous.previous.previous.socket,
                self.state.previous.previous.config,
                async { Some(Ok({
                    let mut config = quinn_proto::ServerConfig::with_crypto(server_config);
                    config.transport_config(self.state.config);
                    Arc::new(config)
                })) },
            ).await
        }))
    }

    /// Gets the server configuration from a future.
    /// 
    /// Useful for when data is being loaded from the filesystem.
    pub fn from_future(
        self,
        future: impl Future<Output = Result<Arc<dyn ServerConfig>, EndpointError>> + Send + Sync + 'static,
    ) -> LoadingEndpoint {
        LoadingEndpoint(self.task_pool.spawn(async move {
            Endpoint::new_inner(
                self.state.previous.previous.previous.socket,
                self.state.previous.previous.config,
                async {
                    let server_config = match future.await {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };

                    let mut config = quinn_proto::ServerConfig::with_crypto(server_config);
                    config.transport_config(self.state.config);
                    return Some(Ok(Arc::new(config)))
                },
            ).await
        }))
    }
}

/// A [`Future`] for the creation of an [`Endpoint`].
/// 
/// This future is automatically run in the background and does not need to be polled by the user.
pub struct LoadingEndpoint(Task<Result<Endpoint, EndpointError>>);

impl Future for LoadingEndpoint {
    type Output = Result<Endpoint, EndpointError>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Future::poll(Pin::new(&mut self.0), cx)
    }
}

/// A reference-counted handle to a QUIC endpoint.
/// 
/// Endpoints manage connections and asynchronously handle I/O.
/// 
/// # Reference-counting
/// As long as an instance of this handle exists, the asynchronous endpoint task will be kept alive and running.
/// When all handles are dropped, the endpoint will shut down. Endpoints can be closed early, without dropping
/// all handles, by using [`close`](Self::close). This frees up most resources until all handles are dropped.
#[derive(Clone)]
pub struct Endpoint(Arc<Handle>);

impl Endpoint {
    async fn new_inner(
        socket: Task<Result<Async<UdpSocket>, std::io::Error>>,
        config: Arc<EndpointConfig>,
        server: impl Future<Output = Option<Result<Arc<quinn_proto::ServerConfig>, EndpointError>>> + Send + Sync + 'static,
    ) -> Result<Endpoint, EndpointError> {
        // Retrieve task pool
        let task_pool = get_task_pool();

        // Zip the futures to run them at the same time
        let (socket, server_config) = futures_lite::future::zip(
            socket,
            server,
        ).await;

        // Unwrap any errors and wrap them in appropriate types
        let socket = Arc::new(socket?);
        let server_config = match server_config {
            Some(Ok(v)) => Some(v),
            Some(Err(e)) => return Err(e),
            None => None,
        };

        // Create channels for communication
        let (io_recv_tx, io_recv_rx) = async_channel::unbounded();
        let (io_send_tx, io_send_rx) = async_channel::unbounded();
        let (conn_event_tx, conn_event_rx) = async_channel::unbounded();
        let (close_signal_tx, close_signal_rx) = async_channel::unbounded();

        // Construct the inner state
        let state = State {
            close_signal_rx,

            io_socket: socket.clone(),

            io_task: task_pool.spawn(io_task(
                socket,
                io_recv_tx,
                io_send_rx
            )),

            io_recv_rx,
            io_send_tx,

            quinn_state: quinn_proto::Endpoint::new(
                config,
                server_config,
                true,
                None,
            ),

            c2e_event_rx: conn_event_rx,
            c2e_event_tx: conn_event_tx,

            connections: HashMap::new(),
        };

        // Start driver task to run in the background
        let driver = task_pool.spawn(driver_task(
            state,
        ));

        // Return shared endpoint thing
        return Ok(Endpoint(Arc::new(Handle {
            driver,

            close_signal_tx,
        })));
    }

    /// Gracefully closes all connections and shuts down the endpoint.
    pub fn close(&self) {
        // We send an event to the state object to shut it down.
        // If there's an error, it means the endpoint is either
        // already closing or closed, so we can safely ignore it.
        let _ = self.0.close_signal_tx.send(CloseSignal {

        });
    }
}

/// An error returned during the creation or execution of an [`Endpoint`].
#[derive(Debug)]
pub enum EndpointError {
    /// An I/O error occurred.
    IoError(std::io::Error),

    /// A TLS error occurred.
    TlsError(rustls::Error),
}

impl From<std::io::Error> for EndpointError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

struct Handle {
    driver: Task<EndpointError>,

    close_signal_tx: Sender<CloseSignal>,
}

struct State {
    close_signal_rx: Receiver<CloseSignal>,

    io_socket: Arc<Async<UdpSocket>>,
    io_task: Task<Result<(), std::io::Error>>,

    io_recv_rx: Receiver<DgramRecv>,
    io_send_tx: Sender<DgramSend>,

    quinn_state: quinn_proto::Endpoint,

    c2e_event_rx: Receiver<(ConnectionHandle, C2EEvent)>,
    c2e_event_tx: Sender<(ConnectionHandle, C2EEvent)>,

    connections: HashMap<ConnectionHandle, HeldConnection>,
}

struct HeldConnection {
    e2c_event_tx: Sender<E2CEvent>,
}

struct CloseSignal {

}

struct DgramRecv {
    origin: SocketAddr,
}

struct DgramSend {
    target: SocketAddr,
}

async fn io_task(
    socket: Arc<Async<UdpSocket>>,
    io_recv_tx: Sender<DgramRecv>,
    io_send_rx: Receiver<DgramSend>,
) -> Result<(), std::io::Error> {
    // TODO: Make this configurable
    let mut scratch = vec![0u8; 2048];

    loop {
        let socket_poller = async {
            match socket.recv_from(&mut scratch[..]).await {
                Ok((length, origin)) => match io_recv_tx.send(DgramRecv {
                    origin,
                }).await {
                    Ok(_) => { /* Do nothing */ },
                    Err(_) => todo!(),
                },

                Err(_) => todo!(),
            }
        };

        let send_poller = async {
            match io_send_rx.recv().await {
                Ok(dgram) => match socket.send_to(
                    todo!(),
                    dgram.target,
                ).await {
                    Ok(_) => { /* Do nothing */ }
                    Err(_) => todo!(),
                },

                Err(_) => todo!(),
            }
        };

        Race::new((
            pin!(socket_poller),
            pin!(send_poller),
        )).await;
    }
}

async fn driver_task(
    state: State,
) -> EndpointError {
    loop {
        let dgram_recv = async {
            match state.io_recv_rx.recv().await {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }
        };

        let conn_events = async {
            match state.c2e_event_rx.recv().await {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }
        };

        Race::new((
            pin!(dgram_recv),
            pin!(conn_events),
        )).await
    }
}