//! Transport layer agnostic config values.

use bevy::prelude::*;

/// Soft limit for connected network clients.
/// Applies across transport layers, though they may choose to ignore it or impose a different limit.
#[derive(Resource)]
pub struct NetworkClientCap(pub u32);