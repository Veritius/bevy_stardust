use std::{io::ErrorKind, sync::Mutex, time::Instant};
use bevy_ecs::{entity::Entities, prelude::*};
use bevy_stardust::prelude::*;
use bytes::BytesMut;
use untrusted::{EndOfInput, Reader};
use crate::{connections::{ConnectionHandleMap, PendingConnections, QuicConnectionBundle}, QuicConnection, QuicEndpoint};

pub(super) fn quic_receive_packets_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    handle_map: Res<ConnectionHandleMap>,
    connections: Query<&QuicConnection>,
    pending: ResMut<PendingConnections>,
    entities: &Entities,
) {
    let pending_mutex = Mutex::new(pending);

    // Receive as many packets as we can
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        let mut pending_local = PendingConnections::default();
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
                                    if let Some((_, _, conn, _)) = pending_local.0.get_mut(&handle) {
                                        crate::polling::handle_connection_event(
                                            handle,
                                            conn.get_mut(),
                                            endpoint.inner.get_mut(),
                                            event,
                                        )
                                    } else {
                                        todo!();
                                    }
                                }
                            },

                            quinn_proto::DatagramEvent::NewConnection(connection) => {
                                pending_local.insert(entities, handle, connection, endpoint_id);
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

        // Incorporate new connections if there are any
        if pending_local.0.len() > 0 {
            pending_mutex.lock().unwrap().incorporate(pending_local);
        }
    });
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