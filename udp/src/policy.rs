//! IP blocking data.

use std::net::IpAddr;
use bevy::prelude::*;

/// IP addresses the UDP transport layer will block.
#[derive(Resource)]
pub struct BlockingPolicy {
    /// Literal IP addresses that will be blocked.
    pub literals: Vec<IpAddr>,
    /// Trait objects that can match against addresses.
    pub rules: Vec<Box<dyn AddressBlockingRule>>,
}

impl BlockingPolicy {
    /// Returns `true` if a literal is matched or a rule blocks the address.
    pub fn is_blocked(&self, address: &IpAddr) -> bool {
        // Match literals
        for literal in &self.literals {
            if literal == address { return true }
        }

        // Match rules
        for rule in &self.rules {
            if rule.is_blocked(address) { return true }
        }

        // The address isn't blocked
        return false
    }
}

/// A rule that can be used to block IPs.
pub trait AddressBlockingRule: Send + Sync {
    /// Returns `true` if `address` is matched by this rule, and therefore should be blocked.
    fn is_blocked(&self, address: &IpAddr) -> bool;
}