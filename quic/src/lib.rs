#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(feature="quiche")))]
compile_error!("You must choose a QUIC implementation.");

#[cfg(feature="quiche")]
mod quiche;

mod backend;
mod bimap;
mod config;
mod connection;
mod crypto;
mod datagrams;
mod endpoint;
mod events;
mod plugin;
mod streams;

pub use config::{AppProtosBuilder, AppProtos, AppProto};
pub use connection::ConnectionShared;
pub use crypto::{PrivateKey, Certificate, CertChain, TrustAnchors, Credentials};
pub use endpoint::{EndpointShared, EndpointBuilder, Client, Server, Dual};
pub use events::TryConnectEvent;
pub use plugin::{QuicPlugin, QuicBackendPlugin, QuicSystems};