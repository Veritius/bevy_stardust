//! Diagnostic tools for Stardust.
//! These are generic, and transport layers may provide better tools for their peers specifically.

mod connections;
mod messages;
mod slowdown;
mod stats;

pub use connections::PeerDiagnosticPlugin;
pub use messages::MessageCountDiagnosticsPlugin;
pub use slowdown::NetworkPerformanceReduction;
pub use stats::PeerStats;