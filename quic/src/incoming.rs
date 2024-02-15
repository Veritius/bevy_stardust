use std::{io::ErrorKind, sync::Mutex, time::Instant};
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use bytes::BytesMut;
use quinn_proto::{Connection, ConnectionEvent, ConnectionHandle};
use untrusted::{EndOfInput, Reader};
use crate::{connections::{ConnectionHandleMap, QuicConnectionBundle}, QuicConnection, QuicEndpoint};

/// Deferred operations coming from [quic_receive_packets_system].
#[derive(Resource, Default)]
pub(crate) struct DeferredReceiveOperations {
    pub new_connections: Mutex<Vec<(ConnectionHandle, Entity, Instant, Connection)>>,
    pub connection_events: Mutex<Vec<(ConnectionHandle, ConnectionEvent)>>,
}

pub(super) fn quic_receive_packets_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    handle_map: Res<ConnectionHandleMap>,
    connections: Query<&QuicConnection>,
    deferred_quic: Res<DeferredReceiveOperations>,
) {
    // Receive as many packets as we can
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        let mut scratch = [0u8; 1472]; // TODO: make this configurable

        loop {
            match endpoint.udp_socket.recv_from(&mut scratch) {
                // Packet received, forward it to the endpoint
                Ok((bytes, address)) => {
                    tracing::trace!("Received a packet of length {bytes} from {address}");
                    if let Some((handle, event)) = endpoint.inner.get_mut().handle(
                        Instant::now(),
                        address,
                        None,
                        None,
                        BytesMut::from(&scratch[..bytes]),
                    ) {
                        match event {
                            // Relay connection events
                            quinn_proto::DatagramEvent::ConnectionEvent(event) => {
                                if let Some(id) = handle_map.0.get(&handle) {
                                    let connection = connections.get(*id).unwrap();
                                    connection.events.lock().unwrap().push(event);
                                } else {
                                    let mut queue = deferred_quic.connection_events.lock().unwrap();
                                    queue.push((handle, event));
                                }
                            },

                            quinn_proto::DatagramEvent::NewConnection(connection) => {
                                let mut queue = deferred_quic.new_connections.lock().unwrap();
                                queue.push((handle, endpoint_id, Instant::now(), connection));
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
    });
}

pub(super) fn apply_deferred_entity_adds_from_incoming_system(
    world: &mut World,
) {
    let mut deferred = world.remove_resource::<DeferredReceiveOperations>().unwrap();
    let mut conn_map = world.remove_resource::<ConnectionHandleMap>().unwrap();

    let new_adds_queue = deferred.new_connections.get_mut().unwrap();
    for (handle, endpoint, added, connection) in new_adds_queue.drain(..) {
        let mut peer_comp = NetworkPeer::new();
        peer_comp.joined = added;

        let entity = world.spawn(QuicConnectionBundle {
            peer_comp,
            quic_comp: QuicConnection::new(endpoint, handle.clone(), connection),
        }).id();

        conn_map.0.insert(handle, entity);
    }

    world.insert_resource(deferred);
    world.insert_resource(conn_map);
}

pub(super) fn apply_deferred_connection_events_from_incoming_system(
    deferred: Res<DeferredReceiveOperations>,
    handle_map: Res<ConnectionHandleMap>,
    mut connections: Query<(Entity, &mut QuicConnection)>,
) {
    let mut def_conn_events = deferred.connection_events.lock().unwrap();
    for (handle, event) in def_conn_events.drain(..) {
        let entity = handle_map.0.get(&handle).unwrap();
        let qres = connections.get_mut(*entity).unwrap();
        let mut events = qres.1.events.lock().unwrap();
        events.push(event);
    }
}

fn read_datagram(
    reader: &mut Reader<'_>,
    channels: &ChannelRegistry,
    cid_len: u8
) -> Result<(ChannelId, u16, Bytes), EndOfInput> {
    // Channel ID
    let cid_bytes = reader.read_bytes(cid_len as usize)?.as_slice_less_safe();
    let cid = ChannelId::from(match cid_len {
        1 => { u32::from_be_bytes([cid_bytes[0], 0, 0, 0]) },
        2 => { u32::from_be_bytes([cid_bytes[0], cid_bytes[1], 0, 0]) }
        3 => { u32::from_be_bytes([cid_bytes[0], cid_bytes[1], cid_bytes[2], 0]) }
        4 => { u32::from_be_bytes(cid_bytes.try_into().unwrap()) }
        0 => panic!(), // Handle this case somehow
        _ => panic!(), // This shouldn't happen
    });

    // Ordering number
    let ordering = {
        let data = channels.get_from_id(cid);
        if data.is_none() { return Err(EndOfInput); } // Make a custom error type for this
        if data.unwrap().ordered != OrderingGuarantee::Unordered {
            let ordering = reader.read_bytes(2)?.as_slice_less_safe();
            u16::from_be_bytes(ordering.try_into().unwrap())
        } else {
            0 // The value returned here is irrelevant since the channel isn't ordered anyway
        }
    };

    // Message payload
    let payload = reader.read_bytes_to_end().as_slice_less_safe();
    let payload = Bytes::from(payload.to_owned());

    Ok((cid, ordering, payload))
}