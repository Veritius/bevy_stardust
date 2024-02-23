//! # bevy_stardust_quic
//! A QUIC transport layer for bevy_stardust.

#![warn(missing_docs)]
#![feature(exclusive_wrapper)]

mod connections;
mod crypto;
mod endpoints;
mod receive;
mod sending;
mod plugin;
mod polling;
mod streams;
mod reading;
mod writing;

pub use plugin::{QuicTransportPlugin, TlsAuthentication};
pub use endpoints::{QuicEndpoint, QuicConnectionManager};
pub use connections::{QuicConnection, ConnectionState};
pub use rustls::{Certificate, PrivateKey, RootCertStore, Error as TlsError};
pub use quinn_proto::TransportConfig;

#[cfg(feature="insecure")]
pub use insecure_pub_uses::*;

mod insecure_pub_uses {
    #![cfg_attr(not(feature="insecure"), allow(unused_imports))]

    use crate::*;
    pub use crypto::ServerCertVerifier;
}