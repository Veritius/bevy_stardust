use std::net::IpAddr;
use bevy::prelude::*;

/// List of IP addresses to block connections from.
#[derive(Resource)]
pub struct BlockingPolicy {
    /// List of addresses to ignore.
    pub addresses: Vec<IpAddr>,
    /// Ignore connections rather than telling them they're blocked.
    pub ignore: bool,
}