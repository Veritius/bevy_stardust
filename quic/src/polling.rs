use bevy::prelude::*;
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn poll_exchange_system(
    mut connections: Query<&mut QuicConnection>,
    mut endpoints: Query<&mut QuicEndpoint>,
) {
    for mut connection_comp in connections.iter_mut() {
        let mut connection = connection_comp.inner.get_mut();
        // connection.
    }
}