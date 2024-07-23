#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(feature="quiche")]
mod quiche;

mod config;
mod connection;
mod crypto;
mod endpoint;
mod events;
mod plugin;

pub mod backend;

pub use config::{AppProtosBuilder, AppProtos, AppProto};
pub use connection::Connection;
pub use crypto::{PrivateKey, Certificate, CertChain, TrustAnchors, Credentials};
pub use endpoint::Endpoint; // {EndpointShared, EndpointBuilder, Client, Server, Dual};
pub use events::TryConnectEvent;
pub use plugin::{QuicPlugin, QuicBackendPlugin, QuicSystems};