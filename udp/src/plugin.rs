use bevy::prelude::*;

/// The UDP transport plugin. Adds the minimal functionality.
/// 
/// For almost all use cases, the `Default` implementation is good enough.
pub struct UdpTransportPlugin {
    /// The number of bytes to use for reliability. Minimum of 1, maximum of 16.
    /// It's best to leave this on 4 unless you know what you're doing.
    pub reliability_bitfield_bytes: u8,
}

impl Default for UdpTransportPlugin {
    fn default() -> Self {
        Self {
            reliability_bitfield_bytes: 4,
        }
    }
}

impl Plugin for UdpTransportPlugin {
    fn build(&self, app: &mut App) {
        // Make sure values are within acceptable ranges
        let reliability_bitfield_bytes = self.reliability_bitfield_bytes.clamp(1, 16);

        // Add the config resource
        app.insert_resource(crate::config::PluginConfig {
            reliability_bitfield_bytes,
        });

        todo!();
    }
}