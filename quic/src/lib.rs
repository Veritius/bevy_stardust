#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod backend;
mod config;
mod ecs;
mod logging;
mod plugin;

pub use plugin::QuicPlugin;

pub use rustls::{
    RootCertStore,
    pki_types::{
        CertificateDer,
        PrivateKeyDer,
    },
};

pub use quinn_proto::{
    EndpointConfig,
    TransportConfig,
    ServerConfig,
    ClientConfig,
};

pub use rustls;
pub use quinn_proto;
pub use ring;