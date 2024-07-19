#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(feature="quiche")]
mod quiche;

mod backend;
mod config;
mod connection;
mod crypto;
mod endpoint;
mod events;
mod plugin;

pub use config::{AppProtosBuilder, AppProtos, AppProto};
pub use connection::ConnectionShared;
pub use crypto::{PrivateKey, Certificate, CertChain, TrustAnchors, Credentials};
pub use endpoint::EndpointShared; // {EndpointShared, EndpointBuilder, Client, Server, Dual};
pub use events::TryConnectEvent;
pub use plugin::{QuicPlugin, QuicBackendPlugin, QuicSystems};