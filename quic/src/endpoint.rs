use std::{future::Future, net::{ToSocketAddrs, UdpSocket}, pin::Pin, sync::Arc};
use async_channel::{Receiver, Sender};
use async_io::Async;
use async_task::Task;
use quinn_proto::{crypto::{HmacKey, ServerConfig}, ConnectionIdGenerator, HashedConnectionIdGenerator, TransportConfig};
use rand::RngCore;
use crate::taskpool::{get_task_pool, NetworkTaskPool};

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
    pub fn use_existing(self, socket: UdpSocket) -> EndpointBuilder<WantsResetKey> {
        let socket = blocking::unblock(move || Async::new(socket));

        EndpointBuilder {
            task_pool: self.task_pool,
            state: WantsResetKey { socket },
        }
    }

    /// Binds to the given address, creating a new socket.
    pub fn bind<A>(self, address: A) -> EndpointBuilder<WantsResetKey>
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
            state: WantsResetKey { socket },
        }
    }
}

/// State for adding a reset key.
pub struct WantsResetKey {
    socket: Task<Result<Async<UdpSocket>, std::io::Error>>,
}

impl EndpointBuilder<WantsResetKey> {
    /// Generates a new reset key from the system's random number generator.
    pub fn generate_new(self) -> EndpointBuilder<WantsCidGenerator> {
        let mut seed = [0; 64];
        rand::thread_rng().fill_bytes(&mut seed);

        EndpointBuilder {
            task_pool: self.task_pool,
            state: WantsCidGenerator {
                previous: self.state,
                reset_key: Arc::new(ring::hmac::Key::new(
                    ring::hmac::HMAC_SHA256,
                    &seed,
                )),
            },
        }
    }

    /// Uses an existing reset key.
    pub fn use_existing(self, reset_key: Arc<dyn HmacKey>) -> EndpointBuilder<WantsCidGenerator> {
        EndpointBuilder {
            task_pool: self.task_pool,
            state: WantsCidGenerator {
                previous: self.state,
                reset_key,
            },
        }
    }
}

/// State for adding a connection ID generator.
pub struct WantsCidGenerator {
    previous: WantsResetKey,
    reset_key: Arc<dyn HmacKey>,
}

impl EndpointBuilder<WantsCidGenerator> {
    /// Uses the default connection ID generator.
    /// 
    /// This is currently [`HashedConnectionIdGenerator`].
    pub fn use_default(self) -> EndpointBuilder<CanBecomeServer> {
        EndpointBuilder {
            task_pool: self.task_pool,
            state: CanBecomeServer {
                previous: self.state,
                cid_generator: Box::new(HashedConnectionIdGenerator::new()),
            },
        }
    }

    /// Uses the suppied connection ID generator.
    pub fn use_existing(self, cid_generator: Box<dyn ConnectionIdGenerator>) -> EndpointBuilder<CanBecomeServer> {
        EndpointBuilder {
            task_pool: self.task_pool,
            state: CanBecomeServer {
                previous: self.state,
                cid_generator,
            },
        }
    }
}

/// State for optionally configuring server behavior.
pub struct CanBecomeServer {
    previous: WantsCidGenerator,
    cid_generator: Box<dyn ConnectionIdGenerator>,
}

impl EndpointBuilder<CanBecomeServer> {
    /// Skips server configuration.
    pub fn client_only(self) -> LoadingEndpoint {
        LoadingEndpoint(self.task_pool.spawn(async move {
            Endpoint::new_inner(
                self.state.previous.previous.socket,
                self.state.previous.reset_key,
                self.state.cid_generator,
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
                self.state.previous.previous.previous.previous.socket,
                self.state.previous.previous.previous.reset_key,
                self.state.previous.previous.cid_generator,
                async { Some(Ok({
                    let mut config = quinn_proto::ServerConfig::with_crypto(server_config);
                    config.transport_config(self.state.config);
                    config
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
                self.state.previous.previous.previous.previous.socket,
                self.state.previous.previous.previous.reset_key,
                self.state.previous.previous.cid_generator,
                async {
                    let server_config = match future.await {
                        Ok(v) => v,
                        Err(e) => return Some(Err(e)),
                    };

                    let mut config = quinn_proto::ServerConfig::with_crypto(server_config);
                    config.transport_config(self.state.config);
                    return Some(Ok(config))
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

/// A reference-counted handle to a QUIC endpoint, handling I/O for [connections](crate::Connection).
#[derive(Clone)]
pub struct Endpoint(Arc<EndpointInner>);

impl Endpoint {
    async fn new_inner(
        socket: Task<Result<Async<UdpSocket>, std::io::Error>>,
        reset_key: Arc<dyn HmacKey>,
        cid_generator: Box<dyn ConnectionIdGenerator>,
        server: impl Future<Output = Option<Result<quinn_proto::ServerConfig, EndpointError>>> + Send + Sync + 'static,
    ) -> Result<Endpoint, EndpointError> {
        todo!()
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

struct EndpointInner {
    io_socket: Arc<Async<UdpSocket>>,
    io_task: Task<Result<(), std::io::Error>>,

    io_recv_rx: Receiver<DgramRecv>,
    io_send_tx: Sender<DgramSend>,
}

async fn io_task(
    socket: Arc<Async<UdpSocket>>,
    io_recv_tx: Sender<DgramRecv>,
    io_send_rx: Receiver<DgramSend>,
) {
    todo!()
}

struct DgramRecv {

}

struct DgramSend {

}