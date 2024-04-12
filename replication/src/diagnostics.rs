//! Diagnostics for replication.

#![allow(missing_docs)]

use bevy::{prelude::*, diagnostic::*};
use crate::prelude::*;

/// Adds diagnostics about replicated entities.
pub struct ReplicatedEntityDiagnosticPlugin;

impl Plugin for ReplicatedEntityDiagnosticPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::TOTAL));

        app.add_systems(Update, entity_diagnostic_system);
    }
}

impl ReplicatedEntityDiagnosticPlugin {
    pub const TOTAL: DiagnosticPath = DiagnosticPath::const_new("net/replicate/entities/total");
}

fn entity_diagnostic_system(
    mut diagnostics: Diagnostics,
    query: Query<(), With<ReplicateEntity>>,
) {
    use ReplicatedEntityDiagnosticPlugin as P;
    diagnostics.add_measurement(&P::TOTAL, || query.iter().count() as f64);
}