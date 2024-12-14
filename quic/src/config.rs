use std::sync::Arc;
use bevy_stardust::channels::ChannelRegistry;

pub use rustls::{self, pki_types::{CertificateDer, PrivateKeyDer}, RootCertStore};

pub use transport::TransportConfig;
pub use endpoint::EndpointConfig;
pub use server::ServerConfig;
pub use client::ClientConfig;

pub mod transport {
    use super::*;

    #[derive(Clone)]
    pub struct TransportConfig {
        quinn: Arc<quinn_proto::TransportConfig>,
    }
}

pub mod endpoint {
    use super::*;

    #[derive(Clone)]
    pub struct EndpointConfig {
        quinn: Arc<quinn_proto::EndpointConfig>,
    }
}

pub mod server {
    use super::*;

    #[derive(Clone)]
    pub struct ServerConfig {
        quinn: Arc<quinn_proto::ServerConfig>,

        channels: Arc<ChannelRegistry>,
    }    
}

pub mod client {
    use super::*;

    #[derive(Clone)]
    pub struct ClientConfig {
        quinn: quinn_proto::ClientConfig,

        channels: Arc<ChannelRegistry>,
    }
}