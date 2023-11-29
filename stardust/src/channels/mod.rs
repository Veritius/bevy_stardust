//! Channel definitions and message storage.

pub mod config;
pub mod extension;
pub mod id;
pub mod registry;

mod incoming;
mod outgoing;

pub(super) fn channels(app: &mut bevy::prelude::App) {
    use bevy::prelude::*;
    use crate::scheduling::*;

    // Channel registry
    app.insert_resource(registry::ChannelRegistry::new());

    // Clearing systems
    app.add_systems(PostUpdate, (incoming::clear_incoming, outgoing::clear_outgoing)
        .after(NetworkWrite::Send).in_set(NetworkWrite::Clear));
}