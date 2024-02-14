use std::{io::ErrorKind, time::Instant};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bytes::BytesMut;
use untrusted::{EndOfInput, Reader};
use crate::{connections::{ConnectionHandleMap, QuicConnectionBundle}, QuicConnection, QuicEndpoint};

pub(super) fn quic_receive_packets_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    handle_map: Res<ConnectionHandleMap>,
    connections: Query<&QuicConnection>,
    commands: ParallelCommands,
) {
    // Receive as many packets as we can
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        let mut scratch = BytesMut::with_capacity(1472); // todo make this configurable

        loop {
            match endpoint.udp_socket.recv_from(&mut scratch) {
                // Packet received, forward it to the endpoint
                Ok((bytes, address)) => {
                    if let Some((handle, event)) = endpoint.inner.get_mut().handle(
                        Instant::now(),
                        address,
                        None,
                        None,
                        scratch.clone(),
                    ) {
                        match event {
                            // Relay connection events
                            quinn_proto::DatagramEvent::ConnectionEvent(event) => {
                                if let Some(id) = handle_map.0.get(&handle) {
                                    let connection = connections.get(*id).unwrap();
                                    connection.events.lock().unwrap().push(event);
                                } else {
                                    todo!();
                                }
                            },

                            // Spawn new connection entities
                            quinn_proto::DatagramEvent::NewConnection(connection) => {
                                commands.command_scope(|mut commands| {
                                    commands.spawn(QuicConnectionBundle {
                                        peer_comp: NetworkPeer::new(),
                                        quic_comp: QuicConnection::new(endpoint_id, handle, connection),
                                    });
                                });
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
                    error!("IO error while reading packets: {e}");
                    break
                }
            }

            // Clear the buffer
            scratch.clear();
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