#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod commands;
mod config;
mod connections;
mod endpoints;
mod plugin;
mod querying;

pub use plugin::QuinnPlugin;
pub use querying::{Connection, Endpoint};
pub use commands::{MakeEndpoint, OpenConnection};

pub use quinn_proto::{self, ClientConfig, ServerConfig, EndpointConfig};
pub use rustls::{self, RootCertStore, pki_types::{CertificateDer, PrivateKeyDer}};