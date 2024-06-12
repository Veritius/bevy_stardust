use bevy::{prelude::*, diagnostic::*};
use crate::prelude::*;

/// Adds diagnostics about how many messages are being sent.
pub struct MessageCountDiagnosticsPlugin;

impl Plugin for MessageCountDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::INCOMING_COUNT));
        app.register_diagnostic(Diagnostic::new(Self::OUTGOING_COUNT));

        app.add_systems(Update, diagnostic_system);
    }
}

impl MessageCountDiagnosticsPlugin {
    /// The number of incoming messages in queues for all peers.
    pub const INCOMING_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/core/messages/outgoing");

    /// The number of outgoing messages in queues for all peers.
    pub const OUTGOING_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/core/messages/outgoing");
}

type QueryFilter = (Or<(With<NetworkMessages<Incoming>>, With<NetworkMessages<Outgoing>>)>, With<NetworkPeer>);

fn diagnostic_system(
    mut diagnostics: Diagnostics,
    query: Query<(Option<&NetworkMessages<Incoming>>, Option<&NetworkMessages<Outgoing>>), QueryFilter>,
) {
    let mut incoming_count: usize = 0;
    let mut outgoing_count: usize = 0;

    for (incoming, outgoing) in query.iter() {
        if let Some(incoming) = incoming {
            incoming_count += incoming.count();
        }

        if let Some(outgoing) = outgoing {
            outgoing_count += outgoing.count();
        }
    }

    diagnostics.add_measurement(&MessageCountDiagnosticsPlugin::INCOMING_COUNT, || incoming_count as f64);
    diagnostics.add_measurement(&MessageCountDiagnosticsPlugin::OUTGOING_COUNT, || outgoing_count as f64);
}