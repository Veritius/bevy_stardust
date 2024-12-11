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

    pub struct TransportConfig {
        inner: quinn_proto::TransportConfig,
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

pub mod endpoint {
    use std::{net::ToSocketAddrs, time::Duration};
    use super::*;

    pub struct EndpointConfig {
        pub(crate) socket: Arc<Async<UdpSocket>>,
        pub(crate) quinn: Arc<quinn_proto::EndpointConfig>,
    }

    impl EndpointConfig {
        pub fn builder() -> EndpointConfigBuilder<WantsSocket> {
            EndpointConfigBuilder {
                state: WantsSocket {
                    _p: (),
                }
            }
        }
    }

    pub struct EndpointConfigBuilder<T> {
        state: T
    }

    pub struct WantsSocket {
        _p: (),
    }

    impl EndpointConfigBuilder<WantsSocket> {
        pub fn bind_address(
            self,
            address: impl ToSocketAddrs,
        ) -> Result<EndpointConfigBuilder<Ready>, io::Error> {
            todo!()
        }

        pub fn with_socket(
            self,
            socket: UdpSocket,
        ) -> Result<EndpointConfigBuilder<Ready>, io::Error> {
            todo!()
        }
    }

    pub struct Ready {
        socket: UdpSocket,
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

pub mod server {
    use super::*;

    pub struct ServerConfig {
        pub(crate) quinn: Arc<quinn_proto::ServerConfig>,
        pub(crate) channels: Arc<ChannelRegistry>,
    }

    impl ServerConfig {
        pub fn builder() -> ServerConfigBuilder<WantsCryptoConfig> {
            ServerConfigBuilder {
                state: WantsCryptoConfig {
                    _p: (),
                }
            }
        }
    }

    pub struct ServerConfigBuilder<T> {
        state: T
    }

    pub struct WantsCryptoConfig {
        _p: (),
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

    pub struct WantsChannelRegistry {
        crypto: Arc<dyn quinn_proto::crypto::ServerConfig>,
    }

    impl ServerConfigBuilder<WantsChannelRegistry> {
        pub fn with_channels(
            self,
            registry: Arc<ChannelRegistry>,
        ) -> ServerConfigBuilder<()> {
            todo!()
        }
    }
}

pub mod client {
    use super::*;

    pub struct ClientConfig {
        pub(crate) remote_address: SocketAddr,
        pub(crate) server_name: Arc<str>,

        pub(crate) quinn: quinn_proto::ClientConfig,
        pub(crate) channels: Arc<ChannelRegistry>,
    }

    impl ClientConfig {
        pub fn builder() -> ClientConfigBuilder<WantsServerDetails> {
            ClientConfigBuilder {
                state: WantsServerDetails {
                    _p: (),
                }
            }
        }
    }

    pub struct ClientConfigBuilder<T> {
        state: T,
    }

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

    pub struct WantsCryptoConfig {
        remote_address: SocketAddr,
        server_name: Arc<str>,

        transport_config: Arc<TransportConfig>,
    }
}