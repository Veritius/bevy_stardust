#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod events;
mod futures;
mod logging;
mod plugin;
mod systems;
mod taskpool;
mod utilities;

pub mod connection;
pub mod endpoint;

pub use connection::{Connection, ConnectionState, ConnectError, ConnectionError};
pub use endpoint::{Endpoint, EndpointWeak, EndpointBuilder, EndpointState, EndpointError};
pub use plugin::QuicPlugin;
pub use taskpool::WorkerThreads;
pub use utilities::EndpointHandler;

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