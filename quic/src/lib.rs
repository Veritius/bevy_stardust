#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod connection;
pub mod endpoint;

mod channels;
mod plugin;
mod runtime;
mod socket;

pub use plugin::QuicPlugin;
pub use endpoint::{Endpoint, EndpointBuilder};
pub use connection::Connection;
pub use runtime::{RuntimeBuilder, Runtime};

pub use rustls;
pub use quinn_proto;

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
    ClientConfig
};