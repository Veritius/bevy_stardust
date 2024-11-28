#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(
    feature="async",
)))]
compile_error!("One of the following features must be enabled: async");

mod commands;
mod config;
mod connection;
mod endpoint;
mod plugin;
mod runtime;

pub use plugin::QuicPlugin;
pub use endpoint::Endpoint;
pub use connection::Connection;
pub use commands::*;
pub use config::*;
pub use runtime::Runtime;

pub use rustls::pki_types::{
    CertificateDer,
    PrivateKeyDer,
};