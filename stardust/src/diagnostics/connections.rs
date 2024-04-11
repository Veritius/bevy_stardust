use bevy::{prelude::*, diagnostic::*};
use crate::prelude::*;

/// Adds diagnostics about various 
pub struct ConnectionDiagnosticPlugin;

impl Plugin for ConnectionDiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::COUNT)
            .with_smoothing_factor(0.0)
            .with_max_history_length(1));

        app.add_systems(Update, diagnostic_system);
    }
}

impl ConnectionDiagnosticPlugin {
    /// Diagnostic path for the amount of connections (entities with [`NetworkPeer`])
    pub const COUNT: DiagnosticPath = DiagnosticPath::const_new("connection_count");
}

fn diagnostic_system(
    mut diagnostics: Diagnostics,
    query: Query<(), With<NetworkPeer>>,
) {
    diagnostics.add_measurement(&ConnectionDiagnosticPlugin::COUNT, || query.iter().count() as f64);
}