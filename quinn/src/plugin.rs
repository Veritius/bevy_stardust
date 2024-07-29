use bevy::prelude::*;
use bevy_stardust::prelude::*;

/// The core plugin for `bevy_stardust_quinn`.
pub struct QuinnPlugin {

}

impl Plugin for QuinnPlugin {
    fn build(&self, app: &mut App) {
        // PreUpdate stage
        app.add_systems(PreUpdate, (
            crate::endpoints::udp_recv_system,
            crate::endpoints::event_exchange_system,
        ).chain().in_set(NetworkRecv::Receive));

        #[cfg(debug_assertions)] {
            app.add_systems(Update, crate::endpoints::safety_check_system);
            app.add_systems(Update, crate::connections::safety_check_system);
        }
    }
}