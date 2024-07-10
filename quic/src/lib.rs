#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(feature="quiche")))]
compile_error!("You must choose a QUIC implementation.");

#[cfg(feature="quiche")]
mod quiche;

mod connection;
mod crypto;
mod datagrams;
mod endpoint;
mod plugin;
mod streams;

pub use connection::Connection;
pub use endpoint::Endpoint;
pub use plugin::QuicPlugin;
pub use crypto::{PrivateKey, Certificate, CertChain, RootCAs, Credentials};