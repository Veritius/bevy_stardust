use bevy::{prelude::*, diagnostic::*};
use crate::prelude::*;

/// Adds diagnostics about connections.
pub struct NetworkPeerDiagnosticPlugin;

impl Plugin for NetworkPeerDiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::COUNT)
            .with_smoothing_factor(0.0)
            .with_max_history_length(1));

        app.add_systems(Update, diagnostic_system);
    }
}

impl NetworkPeerDiagnosticPlugin {
    /// Diagnostic path for the amount of entities with [`NetworkPeer`].
    pub const COUNT: DiagnosticPath = DiagnosticPath::const_new("net/core/peers/total");
}

fn diagnostic_system(
    mut diagnostics: Diagnostics,
    query: Query<(), With<NetworkPeer>>,
) {
    diagnostics.add_measurement(&NetworkPeerDiagnosticPlugin::COUNT, || query.iter().count() as f64);
}