use std::time::Instant;
use bevy_ecs::prelude::*;
use quinn_proto::Transmit;
use quinn_udp::{UdpSocketState, UdpSockRef, UdpState as UdpCapability};
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn quic_poll_transmit_system(
    mut endpoints: Query<&mut QuicEndpoint>,
    mut connections: Query<&mut QuicConnection>,
) {
    // Transmit all packets the endpoint has queued
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        loop {
            match endpoint.inner.get_mut().poll_transmit() {
                Some(transmit) => {
                    let (socket, state, capabilities) = endpoint.socket_io();
                    do_transmit(state, UdpSockRef::from(socket), capabilities, &transmit)
                },
                None => { break },
            }
        }
    });

    // Transmit all packets the connection has queued
    // TODO: This can run in parallel very easily
    connections.par_iter_mut().for_each(|mut connection| {
        let target_endpoint = connection.endpoint.clone();
        let connection = connection.inner.get_mut();
        while let Some(transmit) = connection.poll_transmit(Instant::now(), 64) {
            if let Ok(endpoint) = endpoints.get(target_endpoint) {
                let (socket, state, capabilities) = endpoint.socket_io();
                do_transmit(state, UdpSockRef::from(socket), capabilities, &transmit)
            }
        }
    });
}

fn do_transmit(
    socket_state: &UdpSocketState,
    udp_socket: UdpSockRef,
    capabilities: &UdpCapability,
    transmit: &Transmit,
) {
    match socket_state.send(
        udp_socket,
        capabilities,
        &[map_transmit(transmit)]
    ) {
        Ok(bytes) => { tracing::trace!("Sent a packet of length {bytes} to {}", transmit.destination); },
        Err(e) => { tracing::error!("IO error while sending packet: {e}") },
    }
}

// Map a Transmit from quinn_proto to quinn_udp
fn map_transmit(proto_transmit: &Transmit) -> quinn_udp::Transmit {
    quinn_udp::Transmit {
        destination: proto_transmit.destination,
        ecn: if let Some(ecn) = proto_transmit.ecn {
            quinn_udp::EcnCodepoint::from_bits(ecn as u8)
        } else { None },
        contents: proto_transmit.contents.clone(),
        segment_size: proto_transmit.segment_size,
        src_ip: proto_transmit.src_ip,
    }
}