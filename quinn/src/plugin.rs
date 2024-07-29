use bevy::prelude::*;

/// The core plugin for `bevy_stardust_quinn`.
pub struct QuinnPlugin {

}

impl Plugin for QuinnPlugin {
    fn build(&self, app: &mut App) {

        #[cfg(debug_assertions)] {
            app.add_systems(Update, crate::endpoints::safety_check_system);
            app.add_systems(Update, crate::connections::safety_check_system);
        }
    }
}