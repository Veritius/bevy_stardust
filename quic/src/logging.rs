//! TODO: Remove this module, it's for debugging.

use bevy::prelude::*;
use crate::{QuicConnection, QuicEndpoint};

/// Logs QUIC related information. Mostly for debugging.
pub(crate) fn log_quic_events_system(
    new_endpoints: Query<(Entity, &QuicEndpoint), Added<QuicEndpoint>>,
    new_connections: Query<(Entity, &QuicConnection), Added<QuicConnection>>,
) {
    for (id, comp) in new_endpoints.iter() {
        info!("New endpoint {id:?} with address {} added", comp.udp_socket.local_addr().unwrap());
    }

    for (id, _comp) in new_connections.iter() {
        info!("New connection {id:?} added");
    }
}