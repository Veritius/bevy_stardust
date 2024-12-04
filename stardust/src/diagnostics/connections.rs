use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_diagnostic::*;
use crate::prelude::*;

/// Adds diagnostics about connections.
pub struct PeerDiagnosticPlugin;

impl Plugin for PeerDiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::COUNT)
            .with_smoothing_factor(0.0)
            .with_max_history_length(1));

        app.add_systems(Update, diagnostic_system);
    }
}

impl PeerDiagnosticPlugin {
    /// Diagnostic path for the amount of entities with [`Peer`].
    pub const COUNT: DiagnosticPath = DiagnosticPath::const_new("net/core/peers/total");
}

fn diagnostic_system(
    mut diagnostics: Diagnostics,
    query: Query<(), With<Peer>>,
) {
    diagnostics.add_measurement(&PeerDiagnosticPlugin::COUNT, || query.iter().count() as f64);
}