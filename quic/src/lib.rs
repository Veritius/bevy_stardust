//! # bevy_stardust_quic
//! A QUIC transport layer for bevy_stardust.

#![warn(missing_docs)]
#![feature(exclusive_wrapper)]

mod connections;
mod crypto;
mod endpoints;
mod incoming;
mod logging;
mod misc;
mod outgoing;
mod plugin;
mod polling;

pub use plugin::{QuicTransportPlugin, TlsAuthentication};
pub use endpoints::{QuicEndpoint, QuicConnectionManager};
pub use connections::QuicConnection;
pub use rustls::{Certificate, PrivateKey, RootCertStore, Error as TlsError};

#[cfg(feature="dangerous")]
pub use dangerous_pub_uses::*;

mod dangerous_pub_uses {
    use crate::*;
    pub use crypto::ServerCertVerifier;
}