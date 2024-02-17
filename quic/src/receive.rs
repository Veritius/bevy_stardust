use std::{collections::HashMap, io::{ErrorKind, IoSliceMut}, time::Instant};
use bevy_ecs::prelude::*;
use bytes::BytesMut;
use quinn_udp::{RecvMeta, UdpSockRef};
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn quic_receive_packets_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    connections: Query<&mut QuicConnection>,
    commands: ParallelCommands,
) {

    /*
        This section of code is used as a sanity and safety check so we don't have multiple mutable references to the same object occurring
        Since each Connection is "owned" by one Endpoint entity, we can iterate endpoints in parallel and safely access Components within those parallel tasks
        However, Query doesn't normally allow that, so we have to use the unsafe function 'get_unchecked'. Because of the above, this should be fine.
        We still check that two endpoints don't lay claim to the same connection, just in case. This is a relatively cheap operation considering the syscalls and I/O.
    */

    let mut owned = Vec::with_capacity(connections.iter().len());
    for (_, endpoint) in endpoints.iter() {
        for (_, entity) in &endpoint.connections {
            owned.push(entity.clone());
        }
    }

    owned.sort_unstable();
    let mut last = Entity::PLACEHOLDER;
    for item in &owned {
        if *item == last { panic!("Connection owned by more than one endpoint at a time. Panic because this would cause a data race. Report me immediately!"); }
        last = item.clone();
    }

    drop(owned);

    // I/O starts here :)
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint_component)| {
        let local_address = endpoint_component.local_address().ip();
        let mut pending_local: HashMap<quinn_proto::ConnectionHandle, quinn_proto::Connection> = HashMap::default();

        let mut scratch = [0u8; 1472]; // TODO: make this configurable
        let recv_meta = &mut [RecvMeta::default()];

        let (endpoint, handle_map, socket, state) = endpoint_component.recv_split_borrow();

        loop {
            match state.recv(UdpSockRef::from(socket), &mut [IoSliceMut::new(&mut scratch[..])], recv_meta) {
                // Packet successfully received
                Ok(_) => {
                    // Get packet values
                    let (addr, len) = (recv_meta[0].addr, recv_meta[0].len);
                    let ecn = if let Some(ecn) = recv_meta[0].ecn {
                        // EcnCodepoint is defined seperately in quinn_proto and quinn-udp for some reason,
                        // so we have to convert it manually. It's easy enough, though.
                        quinn_proto::EcnCodepoint::from_bits(ecn as u8)
                    } else { None };
                    tracing::trace!("Received a packet of length {len} from {addr}");

                    // Give the datagram to the Endpoint to handle it
                    if let Some((handle, event)) = endpoint.handle(
                        Instant::now(),
                        addr,
                        Some(local_address.clone()),
                        ecn,
                        BytesMut::from(&scratch[..len])
                    ) { 
                        match event {
                            quinn_proto::DatagramEvent::ConnectionEvent(event) => {
                                if let Some(entity) = handle_map.get(&handle) {
                                    // This is safe because a Connection can only be 'owned' by one Endpoint at a time (we check this earlier too)
                                    // and since we're iterating only endpoints, it's the only mutable reference to the Connection
                                    // TODO: Handle error outcomes without unwrapping/panicking
                                    let mut comp = unsafe { connections.get_unchecked(*entity).unwrap() };
                                    crate::polling::handle_connection_event(endpoint, comp.inner.get_mut(), event);
                                } else {
                                    if let Some(connection) = pending_local.get_mut(&handle) {
                                        // This connection is in thread local storage (just joined)
                                        crate::polling::handle_connection_event(endpoint, connection, event);
                                    } else {
                                        tracing::debug!("Connection event intended for {handle:?}:{endpoint_id:?} was discarded");
                                        continue
                                    }
                                }
                            },
                            quinn_proto::DatagramEvent::NewConnection(connection) => {
                                // Add connection to map
                                pending_local.insert(handle, connection);
                            },
                        }
                    }
                },

                // We've run out of packets to read
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    break
                },

                // Actual IO error
                Err(e) => {
                    tracing::error!("IO error while reading packets: {e}");
                    break
                }
            }
        }

        // Spawn connection entities
        commands.command_scope(|mut commands| {
            for (handle, connection) in pending_local.drain() {
                let remote_address = connection.remote_address();
                let entity = commands.spawn(QuicConnection::new(endpoint_id, handle, connection)).id();
                endpoint_component.connections.insert(handle, entity);
                tracing::info!("New incoming connection {entity:?} ({remote_address}) on endpoint {endpoint_id:?}");
            }
        });
    });
}