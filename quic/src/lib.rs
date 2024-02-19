//! # bevy_stardust_quic
//! A QUIC transport layer for bevy_stardust.

#![warn(missing_docs)]
#![feature(exclusive_wrapper)]

mod connections;
mod crypto;
mod endpoints;
mod receive;
mod misc;
mod sending;
mod plugin;
mod polling;

pub use plugin::{QuicTransportPlugin, TlsAuthentication};
pub use endpoints::{QuicEndpoint, QuicConnectionManager};
pub use connections::QuicConnection;
pub use rustls::{Certificate, PrivateKey, RootCertStore, Error as TlsError};
pub use quinn_proto::TransportConfig;

#[cfg(feature="dangerous")]
pub use dangerous_pub_uses::*;

mod dangerous_pub_uses {
    #![allow(unused_imports)]

    use crate::*;
    pub use crypto::ServerCertVerifier;
}