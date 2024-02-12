use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::{QuicConnection, QuicEndpoint};

pub(super) fn quic_process_outgoing_system(
    mut endpoints: Query<&mut QuicEndpoint>,
    mut connections: Query<&mut QuicConnection>,
    channels: Res<ChannelRegistry>,
    mut reader: NetworkOutgoingReader,
) {
    endpoints.par_iter_mut().for_each(|mut endpoint| {
        loop {
            match endpoint.quic_endpoint.get_mut().poll_transmit() {
                Some(transmit) => {
                    match endpoint.udp_socket.send_to(&transmit.contents, transmit.destination) {
                        Ok(len) => {
                            trace!("Sent a packet of length {len} to {}", transmit.destination);
                        },
                        Err(e) => {
                            error!("IO error while reading packets: {e}");
                            break
                        },
                    }
                },
                None => { break },
            }
        }
    });
}