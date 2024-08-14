//! Various types used to configure replication behaviors.

/// Whether to opt in or out of replication by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplicateOpt {
    /// Do not replicate by default.
    In,

    /// Components are automatically replicated.
    Out,
}