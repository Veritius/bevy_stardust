#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

#[cfg(not(any(feature="quiche")))]
compile_error!("You must choose a QUIC implementation.");

#[cfg(feature="quiche")]
mod quiche;

mod bimap;
mod config;
mod connection;
mod crypto;
mod endpoint;
mod events;
mod plugin;

pub use config::{AppProtosBuilder, AppProtos, AppProto};
pub use connection::Connection;
pub use crypto::{PrivateKey, Certificate, CertChain, TrustAnchors, Credentials};
pub use endpoint::{Endpoint, EndpointBuilder, Client, Server, Dual};
pub use events::TryConnectEvent;
pub use plugin::QuicPlugin;