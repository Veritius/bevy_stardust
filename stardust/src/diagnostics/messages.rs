use bevy::{prelude::*, diagnostic::*};
use crate::prelude::*;

/// Adds diagnostics about how many messages are being sent.
pub struct MessageCountDiagnosticsPlugin;

impl Plugin for MessageCountDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::INCOMING_COUNT));
        app.register_diagnostic(Diagnostic::new(Self::OUTGOING_COUNT));

        app.add_systems(PreUpdate, diagnostic_system::<Incoming>.in_set(NetworkRecv::Synchronise));
        app.add_systems(PostUpdate, diagnostic_system::<Outgoing>.in_set(NetworkSend::Diagnostics));
    }
}

impl MessageCountDiagnosticsPlugin {
    /// The number of incoming messages in queues for all peers.
    pub const INCOMING_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/core/messages/outgoing");

    /// The number of outgoing messages in queues for all peers.
    pub const OUTGOING_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/core/messages/outgoing");
}

fn diagnostic_system<D: MessageDirection>(
    mut diagnostics: Diagnostics,
    query: Query<&PeerMessages<D>, With<Peer>>,
) {
    let count = query.iter().map(|m| m.count()).sum::<usize>() as f64;
    let path = match D::net_dir() {
        NetDirection::Outgoing => MessageCountDiagnosticsPlugin::OUTGOING_COUNT,
        NetDirection::Incoming => MessageCountDiagnosticsPlugin::INCOMING_COUNT,
    };

    diagnostics.add_measurement(&path, || count as f64);
}