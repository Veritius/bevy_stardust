//! # bevy_stardust_quic
//! A QUIC transport layer for bevy_stardust.

#![warn(missing_docs)]
#![feature(exclusive_wrapper)]

mod connections;
mod endpoints;
mod incoming;
mod outgoing;
mod misc;

pub use endpoints::{QuicEndpoint, QuicConnectionManager};
pub use connections::QuicConnection;

use bevy::prelude::*;
use bevy_stardust::scheduling::{NetworkRead, NetworkWrite};

/// Adds QUIC support to Stardust.
pub struct QuicTransportPlugin {
    /// Whether or not to allow self signed certificates.
    /// If set, this makes transport vulnerable to MITM attacks.
    pub allow_self_signed: bool,

    /// The number of reliable streams that are opened.
    /// Higher values reduce head of line blocking.
    pub reliable_streams: u32,

    /// The maximum duration of inactivity that is allowed before a connection is timed out, in seconds.
    /// Set this to something reasonable, like 30 seconds.
    pub timeout_delay: u32,
}

impl Plugin for QuicTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, incoming::quic_process_incoming_system
            .in_set(NetworkRead::Receive));
        app.add_systems(PostUpdate, outgoing::quic_process_outgoing_system
            .in_set(NetworkWrite::Send));

        app.init_resource::<endpoints::SharedEndpointConfig>();
    }
}