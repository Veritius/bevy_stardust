use std::{collections::HashMap, io::{ErrorKind, IoSliceMut}, time::Instant};
use bevy_ecs::prelude::*;
use bytes::BytesMut;
use quinn_udp::{RecvMeta, UdpSockRef};
use crate::{connections::ConnectionHandleMap, QuicConnection, QuicEndpoint};

pub(super) fn quic_receive_packets_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    handle_map: Res<ConnectionHandleMap>,
    connections: Query<&QuicConnection>,
    commands: ParallelCommands,
) {
    // Receive as many packets as we can
    endpoints.par_iter_mut().for_each(|(endpoint_id, mut endpoint)| {
        let local_address = endpoint.local_address().ip();
        let mut pending_local: HashMap<quinn_proto::ConnectionHandle, quinn_proto::Connection> = HashMap::default();

        let mut scratch = [0u8; 1472]; // TODO: make this configurable
        let recv_meta = &mut [RecvMeta::default()];

        let (endpoint, socket, state, capabilities) = endpoint.socket_io_with_endpoint();

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

                    // Hand off the datagram to the Endpoint
                    if let Some((handle, event)) = endpoint.handle(
                        Instant::now(),
                        addr,
                        Some(local_address.clone()),
                        ecn,
                        BytesMut::from(&scratch[..len])
                    ) {
                        todo!()
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
                commands.spawn(QuicConnection::new(endpoint_id, handle, connection));
            }
        });
    });
}