use std::{io, net::{SocketAddr, UdpSocket}, sync::Arc};
use async_io::Async;
use bevy_stardust::channels::ChannelRegistry;

pub use rustls::{
    self,
    RootCertStore,
    pki_types::{
        CertificateDer,
        PrivateKeyDer,
    },
};

pub use transport::TransportConfig;
pub use endpoint::EndpointConfig;
pub use server::ServerConfig;
pub use client::ClientConfig;

mod transport {
    use std::time::Duration;

    /// Transport parameters for QUIC.
    pub struct TransportConfig {
        pub(crate) inner: quinn_proto::TransportConfig,
    }

    impl Default for TransportConfig {
        fn default() -> Self {
            Self {
                inner: quinn_proto::TransportConfig::default(),
            }
        }
    }

    impl TransportConfig {
        pub fn allow_spin_bit(&mut self, value: bool) -> &mut Self {
            self.inner.allow_spin(value);
            return self;
        }

        pub fn dgram_send_buf_size(&mut self, value: usize) -> &mut Self {
            self.inner.datagram_send_buffer_size(value);
            return self;
        }

        pub fn set_initial_mtu(&mut self, value: u16) -> &mut Self {
            self.inner.initial_mtu(value.max(1200));
            return self;
        }

        pub fn set_keep_alive_interval(&mut self, value: Option<Duration>) -> &mut Self {
            self.inner.keep_alive_interval(value);
            return self;
        }

        pub fn set_packet_loss_threshold(&mut self, value: u32) -> &mut Self {
            self.inner.packet_threshold(value.max(3));
            return self;
        }

        pub fn set_receive_window(&mut self, value: u64) -> &mut Self {
            let value = value.min((2u64.pow(62))-1);
            let value = quinn_proto::VarInt::from_u64(value).unwrap();
            self.inner.receive_window(value);
            return self;
        }

        pub fn set_transmit_window(&mut self, value: u64) -> &mut Self {
            self.inner.send_window(value);
            return self;
        }
    }
}

/// Error types for configuration.
pub mod error {
    pub struct InvalidInitialCypherSuite;
}

/// Endpoint configuration.
pub mod endpoint {
    use std::{net::ToSocketAddrs, time::Duration};
    use super::*;

    /// Endpoint configuration.
    pub struct EndpointConfig {
        pub(crate) socket: Arc<Async<UdpSocket>>,
        pub(crate) quinn: Arc<quinn_proto::EndpointConfig>,
    }

    impl EndpointConfig {
        /// Returns a builder to create the configuration value.
        pub fn builder() -> EndpointConfigBuilder<WantsSocket> {
            EndpointConfigBuilder {
                state: WantsSocket {
                    _p: (),
                }
            }
        }
    }

    /// A builder for [`EndpointConfig`].
    pub struct EndpointConfigBuilder<T> {
        state: T
    }

    /// State for adding a UDP socket.
    pub struct WantsSocket {
        _p: (),
    }

    impl EndpointConfigBuilder<WantsSocket> {
        /// Binds a new UDP socket to use with the endpoint.
        ///
        /// # Warning
        /// This operation will block for a substantial amount of time due to interacting with the OS.
        /// This may cause issues such as stutters and general unexpected delays in a game tick.
        /// To avoid this, it's recommended to use the `blocking` crate or to create a thread.
        pub fn bind_address(
            self,
            address: impl ToSocketAddrs,
        ) -> Result<EndpointConfigBuilder<Ready>, io::Error> {
            self.with_socket(UdpSocket::bind(address)?)
        }

        /// Uses an existing UDP socket with the endpoint.
        ///
        /// # Warning
        /// This operation will block for a substantial amount of time due to interacting with the OS.
        /// This may cause issues such as stutters and general unexpected delays in a game tick.
        /// To avoid this, it's recommended to use the `blocking` crate or to create a thread.
        pub fn with_socket(
            self,
            socket: UdpSocket,
        ) -> Result<EndpointConfigBuilder<Ready>, io::Error> {
            Ok(EndpointConfigBuilder { state: Ready {
                socket: Arc::new(Async::new(socket)?),
                config: quinn_proto::EndpointConfig::default(),
            } })
        }
    }

    /// Ready to complete, some additional configuration allowed.
    pub struct Ready {
        socket: Arc<Async<UdpSocket>>,
        config: quinn_proto::EndpointConfig,
    }

    impl EndpointConfigBuilder<Ready> {
        pub fn set_max_udp_payload_size(mut self, value: u16) -> Self {
            self.state.config.max_udp_payload_size(value.max(1200)).unwrap();
            return self;
        }

        pub fn set_supported_versions(mut self, versions: Vec<u32>) -> Self {
            self.state.config.supported_versions(versions);
            return self;
        }

        pub fn allow_quic_grease_bit(mut self, value: bool) -> Self {
            self.state.config.grease_quic_bit(value);
            return self;
        }

        pub fn set_min_reset_interval(mut self, value: Duration) -> Self {
            self.state.config.min_reset_interval(value);
            return self;
        }
    }

    impl EndpointConfigBuilder<Ready> {
        pub fn build(self) -> EndpointConfig {
            todo!()
        }
    }
}

/// Server configuration, for incoming connections.
pub mod server {
    use super::*;

    /// Server configuration. Required to accept incoming connections.
    pub struct ServerConfig {
        pub(crate) quinn: Arc<quinn_proto::ServerConfig>,
        pub(crate) channels: Arc<ChannelRegistry>,
    }

    impl ServerConfig {
        /// Returns a builder to create the configuration value.
        pub fn builder() -> ServerConfigBuilder<WantsTransportConfig> {
            ServerConfigBuilder {
                state: WantsTransportConfig {
                    _p: (),
                }
            }
        }
    }

    /// A builder for [`ServerConfig`].
    pub struct ServerConfigBuilder<T> {
        state: T
    }

    /// State for adding associated [`TransportConfig`] values.
    pub struct WantsTransportConfig {
        _p: (),
    }

    impl ServerConfigBuilder<WantsTransportConfig> {
        pub fn with_transport_config(
            self,
            transport_config: TransportConfig,
        ) -> ServerConfigBuilder<WantsCryptoConfig> {
            ServerConfigBuilder { state: WantsCryptoConfig {
                transport_config,
            } }
        }
    }

    /// State for adding cryptographic/TLS configuration.
    pub struct WantsCryptoConfig {
        transport_config: TransportConfig,
    }

    impl ServerConfigBuilder<WantsCryptoConfig> {
        pub fn with_single_cert(
            self,
            cert_chain: Vec<CertificateDer<'static>>,
            key: PrivateKeyDer<'static>,
        ) -> Result<ServerConfigBuilder<WantsChannelRegistry>, rustls::Error> {
            todo!()
        }
    }

    /// State for adding a Stardust channel registry.
    pub struct WantsChannelRegistry {
        transport_config: TransportConfig,
        crypto: Arc<dyn quinn_proto::crypto::ServerConfig>,
    }

    impl ServerConfigBuilder<WantsChannelRegistry> {
        pub fn with_channels(
            self,
            registry: Arc<ChannelRegistry>,
        ) -> ServerConfigBuilder<Ready> {
            ServerConfigBuilder { state: Ready {
                transport_config: self.state.transport_config,
                crypto: self.state.crypto,
                registry,
            } }
        }
    }

    /// Ready to complete, some additional configuration allowed.
    pub struct Ready {
        transport_config: TransportConfig,
        crypto: Arc<dyn quinn_proto::crypto::ServerConfig>,
        registry: Arc<ChannelRegistry>,
    }

    impl ServerConfigBuilder<Ready> {
        pub fn build(self) -> ServerConfig {
            let mut quinn = quinn_proto::ServerConfig::with_crypto(self.state.crypto);
            quinn.transport_config(self.state.transport_config.inner.into());

            ServerConfig {
                quinn: Arc::new(quinn),
                channels: self.state.registry,
            }
        }
    }
}

/// Client configuration, for outgoing connections.
pub mod client {
    use super::*;

    /// Client configuration. Required to create outgoing connections.
    pub struct ClientConfig {
        pub(crate) remote_address: SocketAddr,
        pub(crate) server_name: Arc<str>,

        pub(crate) quinn: quinn_proto::ClientConfig,
        pub(crate) channels: Arc<ChannelRegistry>,
    }

    impl ClientConfig {
        /// Returns a builder to create the configuration value.
        pub fn builder() -> ClientConfigBuilder<WantsServerDetails> {
            ClientConfigBuilder {
                state: WantsServerDetails {
                    _p: (),
                }
            }
        }
    }

    /// A builder for [`ClientConfig`].
    pub struct ClientConfigBuilder<T> {
        state: T,
    }

    /// A state for adding host details.
    pub struct WantsServerDetails {
        _p: (),
    }

    impl ClientConfigBuilder<WantsServerDetails> {
        pub fn with_server_details(
            self,
            remote_address: SocketAddr,
            server_name: impl Into<Arc<str>>,
        ) -> ClientConfigBuilder<WantsTransportConfig> {
            ClientConfigBuilder { state: WantsTransportConfig {
                remote_address,
                server_name: server_name.into(),
            } }
        }
    }

    /// A state for adding associated [`TransportConfig`] values.
    pub struct WantsTransportConfig {
        remote_address: SocketAddr,
        server_name: Arc<str>,
    }

    impl WantsTransportConfig {
        pub fn with_transport_config(
            self,
            transport_config: Arc<TransportConfig>,
        ) -> ClientConfigBuilder<WantsCryptoConfig> {
            ClientConfigBuilder { state: WantsCryptoConfig {
                remote_address: self.remote_address,
                server_name: self.server_name,
                transport_config,
            } }
        }
    }

    /// A state for adding cryptographic/TLS configuration.
    pub struct WantsCryptoConfig {
        remote_address: SocketAddr,
        server_name: Arc<str>,

        transport_config: Arc<TransportConfig>,
    }

    impl ClientConfigBuilder<WantsCryptoConfig> {
        pub fn with_tls_config(
            self,
            config: Arc<rustls::ClientConfig>,
            suite: rustls::quic::Suite,
        ) -> Result<ClientConfigBuilder<Ready>, error::InvalidInitialCypherSuite> {
            Ok(ClientConfigBuilder { state: Ready {
                remote_address: self.state.remote_address,
                server_name: self.state.server_name,
                transport_config: self.state.transport_config,
                crypto_config: Arc::new(quinn_proto::crypto::rustls::QuicClientConfig::with_initial(
                    config,
                    suite,
                ).map_err(|_| error::InvalidInitialCypherSuite)?),
                quic_version: 1,
            } })
        }
    }

    /// Ready to complete, some additional configuration allowed.
    pub struct Ready {
        remote_address: SocketAddr,
        server_name: Arc<str>,

        transport_config: Arc<TransportConfig>,
        crypto_config: Arc<dyn quinn_proto::crypto::ClientConfig>,
        quic_version: u32,
    }

    impl ClientConfigBuilder<Ready> {
        pub fn set_quic_version(&mut self, version: u32) -> &mut Self {
            self.state.quic_version = version;
            return self;
        }

        pub fn build(self) -> ClientConfig {
            todo!()
        }
    }
}