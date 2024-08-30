#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod connections;
mod endpoints;
mod manager;
mod plugin;
mod querying;

pub use manager::Endpoints;
pub use plugin::QuinnPlugin;

pub use quinn_proto::{self, ClientConfig, ServerConfig, EndpointConfig};
pub use rustls::{self, RootCertStore, pki_types::{CertificateDer, PrivateKeyDer}};