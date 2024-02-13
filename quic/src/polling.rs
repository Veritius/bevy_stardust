use std::time::{Duration, Instant};
use bevy::prelude::*;
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn event_exchange_polling_system(
    mut connections: Query<&mut QuicConnection>,
    mut endpoints: Query<&mut QuicEndpoint>,
) {
    // This can't be in parallel because we access the endpoints query
    for mut connection_comp in connections.iter_mut() {
        let connection_handle = connection_comp.handle.clone();
        let target_endpoint = connection_comp.endpoint.clone();
        let connection = connection_comp.inner.get_mut();

        // Handle timeouts
        if let Some(timeout) = connection.poll_timeout() {
            if timeout.duration_since(Instant::now()) == Duration::ZERO {
                connection.handle_timeout(Instant::now());
            }
        }

        // Handle endpoint and connection events
        let mut endpoint = endpoints.get_mut(target_endpoint).unwrap();
        while let Some(event) = connection.poll_endpoint_events() {
            if let Some(event) = endpoint.inner.get_mut().handle_event(connection_handle, event) {
                connection.handle_event(event);
            }
        }
    }
}

pub(super) fn connection_events_polling_system(
    mut connections: Query<&mut QuicConnection>
) {
    connections.par_iter_mut().for_each(|mut connection_comp| {
        let connection = connection_comp.inner.get_mut();
        while let Some(event) = connection.poll() {
            todo!();
        }
    });
}