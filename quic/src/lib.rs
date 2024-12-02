#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(
    feature="async",
)))]
compile_error!("One of the following features must be enabled: async");

pub mod connection;
pub mod endpoint;

mod config;
mod plugin;
mod runtime;

pub use plugin::QuicPlugin;
pub use endpoint::{Endpoint, EndpointBuilder};
pub use connection::Connection;
pub use config::*;
pub use runtime::Runtime;

pub use rustls::pki_types::{
    CertificateDer,
    PrivateKeyDer,
};