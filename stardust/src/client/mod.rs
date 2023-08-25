//! Code for writing

pub mod connection;
pub mod receive;
pub mod send;
pub mod peers;

mod systems;

use bevy::prelude::*;
use self::connection::RemoteConnectionStatus;

/// Add functionality that allows the app to work only as a client.
pub(crate) fn build_dedi_client(app: &mut App) {
    app.add_state::<RemoteConnectionStatus>();
}