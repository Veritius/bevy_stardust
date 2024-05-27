//! Diagnostics for the transport layer.

#![allow(missing_docs)]

use bevy::{prelude::*, diagnostic::*};
use crate::{prelude::*, schedule::PostUpdateSet};

/// Adds diagnostics about [`Endpoint`]s.
pub struct EndpointDiagnosticsPlugin;

impl EndpointDiagnosticsPlugin {
    pub const ENDPOINT_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/udp/endpoints/total");
    pub const PACKETS_SENT_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/udp/endpoints/packets_sent");
    pub const PACKETS_RECV_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/udp/endpoints/packets_recv");
    pub const BYTES_SENT_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/udp/endpoints/bytes_sent");
    pub const BYTES_RECV_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/udp/endpoints/bytes_recv");
}

impl Plugin for EndpointDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::ENDPOINT_COUNT)
            .with_max_history_length(1)
            .with_smoothing_factor(0.0));

        app.register_diagnostic(Diagnostic::new(Self::PACKETS_SENT_COUNT));
        app.register_diagnostic(Diagnostic::new(Self::PACKETS_RECV_COUNT));
        app.register_diagnostic(Diagnostic::new(Self::BYTES_SENT_COUNT));
        app.register_diagnostic(Diagnostic::new(Self::BYTES_RECV_COUNT));

        app.add_systems(PostUpdate, endpoint_diagnostics_system
            .in_set(PostUpdateSet::UpdateStatistics));
    }
}

fn endpoint_diagnostics_system(
    mut diagnostics: Diagnostics,
    endpoints: Query<&Endpoint>,
) {
    let mut endpoint_count: usize = 0;
    let mut packet_send_total: usize = 0;
    let mut packet_recv_total: usize = 0;
    let mut bytes_send_total: usize = 0;
    let mut bytes_recv_total: usize = 0;

    for endpoint in endpoints.iter() {
        endpoint_count += 1;

        let statistics = &endpoint.statistics;
        packet_send_total += statistics.tick_packets_sent as usize;
        packet_recv_total += statistics.tick_packets_received as usize;
        bytes_send_total += statistics.tick_bytes_sent as usize;
        bytes_recv_total += statistics.tick_bytes_received as usize;
    }

    use self::EndpointDiagnosticsPlugin as P;
    diagnostics.add_measurement(&P::ENDPOINT_COUNT, || endpoint_count as f64);
    diagnostics.add_measurement(&P::PACKETS_SENT_COUNT, || packet_send_total as f64);
    diagnostics.add_measurement(&P::PACKETS_RECV_COUNT, || packet_recv_total as f64);
    diagnostics.add_measurement(&P::BYTES_SENT_COUNT, || bytes_send_total as f64);
    diagnostics.add_measurement(&P::BYTES_RECV_COUNT, || bytes_recv_total as f64);
}

/// Adds diagnostics about [`Endpoint`]s.
pub struct ConnectionDiagnosticsPlugin;

impl ConnectionDiagnosticsPlugin {
    pub const CONNECTION_COUNT: DiagnosticPath = DiagnosticPath::const_new("net/udp/connections/total");
}

impl Plugin for ConnectionDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::CONNECTION_COUNT)
            .with_max_history_length(1)
            .with_smoothing_factor(0.0));

        app.add_systems(PostUpdate, connection_diagnostics_system
            .in_set(PostUpdateSet::UpdateStatistics));
    }
}

fn connection_diagnostics_system(
    mut diagnostics: Diagnostics,
    connections: Query<&Connection>,
) {
    let connection_count = connections.iter().count();

    use self::ConnectionDiagnosticsPlugin as P;
    diagnostics.add_measurement(&P::CONNECTION_COUNT, || connection_count as f64);
}