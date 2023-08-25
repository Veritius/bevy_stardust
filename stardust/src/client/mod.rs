//! Client-side code.

pub mod prelude;

pub mod connection;
pub mod receive;
pub mod send;
pub mod peers;

mod systems;

use bevy::prelude::*;
use self::connection::RemoteConnectionStatus;

pub(crate) fn build_dedi_client(app: &mut App) {
    app.add_state::<RemoteConnectionStatus>();
}