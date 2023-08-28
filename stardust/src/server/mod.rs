//! Server-side code.

pub mod clients;
pub mod receive;
pub mod send;
pub mod connection;
pub mod settings;

use bevy::prelude::App;
use self::connection::*;

/// Add functionality that allows the app to work as a server.
pub(in crate) fn build_server(app: &mut App) {
    app.add_event::<TryDisconnectEvent>();
    app.add_event::<PlayerConnectedEvent>();
    app.add_event::<PlayerDisconnectedEvent>();
}