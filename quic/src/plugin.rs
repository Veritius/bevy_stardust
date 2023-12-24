//! The QUIC transport layer plugin.

use bevy::prelude::*;

/// Adds a QUIC transport layer to the `App`.
pub struct QuicTransportPlugin {
    /// If enabled, the transport layer will permit acting as:
    /// - More than one client at once
    /// - More than one server at once
    /// - A client and a server at once
    /// 
    /// Disabled by default. Most games shouldn't need this.
    pub allow_multipurpose: bool,

    /// Allow clients to migrate to new IP addresses.
    /// When running a server, improves behavior for clients that move between different internet connections or suffer NAT rebinding.
    /// Enabled by default.
    /// 
    /// See [Section 9 of IETF RFC 9000](https://www.rfc-editor.org/rfc/rfc9000.html#name-connection-migration) for more information.
    pub allow_migration: bool,

    /// Default maximum concurrent connections for a single endpoint.
    /// Applies individually to each endpoint.
    pub concurrent_connections: u32,

    /// Maximum size of UDP payloads. This improves network performance at the cost of higher memory usage. 
    /// This value must be at least 1200, and defaults to 1472, the largest MTU almost all connections will accept.
    pub max_payload_size: u16,
}

impl Plugin for QuicTransportPlugin {
    fn build(&self, _app: &mut App) {
        todo!()
    }
}