//! Replication groups.

use crate::config::Clusivity;

/// A replication group, allowing configuration to be applied to many peers at once.
pub struct ReplicationGroup {
    clusivity: Clusivity,
}