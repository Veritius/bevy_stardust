//! Diagnostic tools for Stardust.
//! These are generic, and transport layers may provide better tools for their peers specifically.

mod connections;
mod messages;

pub use connections::PeerDiagnosticPlugin;
pub use messages::MessageCountDiagnosticsPlugin;