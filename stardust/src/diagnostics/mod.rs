//! Diagnostic tools for Stardust.
//! These are generic, and transport layers may provide better tools for their peers specifically.

mod connections;
mod messages;
mod slowdown;

pub use connections::NetworkPeerDiagnosticPlugin;
pub use messages::MessageCountDiagnosticsPlugin;
pub use slowdown::NetworkPerformanceReduction;