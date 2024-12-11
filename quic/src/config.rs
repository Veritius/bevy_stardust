use std::{net::{SocketAddr, UdpSocket}, sync::Arc};
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
}