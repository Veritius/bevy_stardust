use std::{io, net::{SocketAddr, UdpSocket}, sync::Arc};
use async_io::Async;
use bevy_stardust::channels::ChannelRegistry;

pub use rustls::{
    RootCertStore,
    pki_types::{
        CertificateDer,
        PrivateKeyDer,
    },
};

pub use rustls;
pub use quinn_proto;
pub use ring;

pub use endpoint::EndpointConfig;
pub use server::ServerConfig;
pub use client::ClientConfig;

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
        ) -> ClientConfigBuilder<WantsCryptoConfig> {
            todo!()
        }
    }

    pub struct WantsCryptoConfig {
        remote_address: SocketAddr,
        server_name: Arc<str>,
    }
}