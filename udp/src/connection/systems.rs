use std::collections::BTreeSet;
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{endpoint::ConnectionOwnershipToken, plugin::PluginConfiguration, prelude::*};
use super::{machine::*, PotentialNewPeer};

pub(crate) fn potential_new_peers_system(
    mut commands: Commands,
    mut events: EventReader<PotentialNewPeer>,
    mut endpoints: Query<&mut Endpoint>,
    existing: Query<&Connection>,
) {
    // Map of all addresses we recognise
    // TODO: Cache this value in a Local?
    let mut known = existing.iter()
    .map(|x| x.inner.shared.remote_address())
    .collect::<BTreeSet<_>>();

    // Read all events
    for event in events.read() {
        // Don't add peers we're already aware of
        if known.contains(&event.address) { continue }

        // Register our awareness of this peer
        // This is necessary so bunched-up packets don't
        // spawn multiple entities for the same real-world machine
        known.insert(event.address);

        // Create connection component
        let mut comp = Connection::new(
            event.endpoint,
            event.address,
            ConnectionDirection::Server,
        );

        // Put the payload into the receive queue of the connection
        comp.inner_mut().shared.recv_queue.push_back(event.payload.clone());

        // Insert connection into the world
        let entity = commands.spawn(comp).id();

        // Register the new connection to the endpoint
        match endpoints.get_mut(event.endpoint) {
            Ok(mut endpoint) => {
                // SAFETY: Spawn on Commands generates a new entity ID concurrently
                let token = unsafe { ConnectionOwnershipToken::new(entity) };
                endpoint.connections.insert(event.address, token);
            },
            Err(_) => {
                warn!("Endpoint {:?} was despawned, but a peer was about to connect to it.", event.endpoint);
            },
        };
    }
}

pub(crate) fn connection_preupdate_ticking_system(
    config: Res<PluginConfiguration>,
    registry: Res<ChannelRegistry>,
    mut connections: Query<(Entity, &mut Connection, Option<&mut NetworkMessages<Incoming>>)>,
) {
    // Tick all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, messages)| {
        // Tracing stuff
        let trace_span = trace_span!("Running preupdate tick", peer=?entity);
        let _entered = trace_span.entered();

        // Run tick function
        let inner = connection.inner_mut();
        inner.machine.tick_preupdate(&mut inner.shared, PreUpdateTickData {
            config: &config,
            registry: &registry,
            messages,
        });
    });
}

pub(crate) fn connection_postupdate_ticking_system(
    config: Res<PluginConfiguration>,
    registry: Res<ChannelRegistry>,
    mut connections: Query<(Entity, &mut Connection, Option<Ref<NetworkMessages<Outgoing>>>)>,
) {
    // Tick all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, messages)| {
        // Tracing stuff
        let trace_span = trace_span!("Running postupdate tick", peer=?entity);
        let _entered = trace_span.entered();

        // Run tick function
        let inner = connection.inner_mut();
        inner.machine.tick_postupdate(&mut inner.shared, PostUpdateTickData {
            config: &config,
            registry: &registry,
            messages,
        });
    });
}

// TODO: Use change detection.
pub(crate) fn close_connections_system(
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
    connections: Query<(Entity, &Connection), Changed<Connection>>,
) {
    // This doesn't need to be in parallel.
    for (entity, connection) in connections.iter() {
        let connection = connection.inner();
        if connection.state() == ConnectionState::Closed {
            // Despawn entity
            commands.entity(entity).despawn();

            // Remove from the connection map
            if let Ok(mut endpoint) = endpoints.get_mut(connection.shared.owning_endpoint()) {
                endpoint.remove_peer(entity);
            }
        }
    }
}