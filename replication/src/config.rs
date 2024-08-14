//! Various types used to configure replication behaviors.

/// Inclusivity and exclusivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clusivity {
    /// Exclude by default.
    In,

    /// Include by default.
    Out,
}