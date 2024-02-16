use std::{net::{SocketAddr, UdpSocket}, time::Instant};

use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn quic_process_outgoing_system(
    mut endpoints: Query<&mut QuicEndpoint>,
    mut connections: Query<&mut QuicConnection>,
) {
    // Transmit all packets the endpoint has queued
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        loop {
            match endpoint.inner.get_mut().poll_transmit() {
                Some(transmit) => {
                    send_packet(
                        &endpoint.udp_socket,
                        &transmit.contents,
                        transmit.destination
                    );                    
                },
                None => { break },
            }
        }
    });

    // Transmit all packets the connection has queued
    connections.par_iter_mut().for_each(|mut connection_comp| {
        let target_endpoint = connection_comp.endpoint.clone();
        let connection = connection_comp.inner.get_mut();
        while let Some(transmit) = connection.poll_transmit(Instant::now(), 64) {
            // Get the UDP socket and queue a send
            if let Ok(endpoint) = endpoints.get(target_endpoint) {
                send_packet(
                    &endpoint.udp_socket,
                    &transmit.contents,
                    transmit.destination
                );
            }
        }
    });
}

fn send_packet(socket: &UdpSocket, payload: &[u8], address: SocketAddr) {
    match socket.send_to(payload, address) {
        Ok(len) => {
            assert_eq!(payload.len(), len); // this should not be different
            tracing::trace!("Sent a packet of length {len} to {address}");
        },
        Err(e) => {
            tracing::error!("IO error while sending packet of length {} to {address}: {e}", payload.len());
        },
    }
}