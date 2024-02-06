//! The QUIC transport layer plugin.

use bevy::prelude::*;
use bevy_stardust::scheduling::*;
use crate::{reader::quic_reader_system, writer::quic_writer_system};

/// Adds a QUIC transport layer to the `App`.
pub struct QuicTransportPlugin {
    /// If enabled, the transport layer will permit acting as:
    /// - More than one client at once
    /// - More than one server at once
    /// - A client and a server at once
    /// 
    /// Most games do not need this functionality.
    /// If in doubt, set to `false`.
    pub allow_multipurpose: bool,
}

impl Plugin for QuicTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, quic_reader_system
            .in_set(NetworkRead::Receive));
        app.add_systems(PostUpdate, quic_writer_system
            .in_set(NetworkWrite::Send));
    }
}