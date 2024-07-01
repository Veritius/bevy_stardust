#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod connections;
mod datagrams;
mod endpoints;
mod framing;
mod plugin;
mod streams;

pub use plugin::*;
pub use connections::QuicConnection;
pub use endpoints::QuicEndpoint;
pub use quinn_proto::{EndpointConfig, ClientConfig, ServerConfig};
pub use quinn_proto::crypto::rustls::{QuicClientConfig, QuicServerConfig};
pub use rustls::pki_types::{CertificateDer, PrivateKeyDer};
pub use rustls::{ServerConfig as TlsServerConfig, ClientConfig as TlsClientConfig};