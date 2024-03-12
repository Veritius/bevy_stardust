use std::io::ErrorKind;
use bevy_ecs::prelude::*;
use bytes::Bytes;
use crate::{packet::IncomingPacket, Connection, Endpoint};

// Receives packets from UDP sockets
pub(crate) fn io_receiving_system(
    endpoints: Query<&mut Endpoint>,
    connections: Query<&mut Connection>,
) {
    // Iterate all endpoints
    endpoints.par_iter().for_each(|endpoint| {
        loop {
            let mut scratch = [0u8; 1478];
            match endpoint.socket.recv_from(&mut scratch) {
                // Received a UDP packet
                Ok((bytes, origin)) => {
                    match endpoint.connections.get(&origin) {
                        // We know this peer
                        Some(token) => {
                            // SAFETY: This is fine because of ConnectionOwnershipToken's guarantees
                            let mut connection = unsafe { connections.get_unchecked(token.inner()).unwrap() };

                            // We append it to the queue for later processing
                            connection.packet_queue.push_incoming(IncomingPacket {
                                payload: Bytes::copy_from_slice(&scratch[..bytes]),
                            });
                        },

                        // We don't know this peer
                        None => {
                            todo!()
                        },
                    }
                },

                // No more packets to read
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    // Break out of the loop
                    break
                },

                // I/O error reported by the system
                Err(err) => {
                    // TODO: Close endpoints based on certain errors
                    todo!();
                }
            }
        }
    });
}

// Processes packets into individual messages
pub(crate) fn packet_parsing_system(
    mut connections: Query<&mut Connection>,
) {
    use untrusted::*;

    // Parses things in parallel
    connections.par_iter_mut().for_each(|mut connection| {
        // Iterate all packets this peer has received
        while let Some(packet) = connection.packet_queue.pop_incoming() {
            let mut reader = Reader::new(Input::from(&packet.payload));

            // Wrap in a closure so we can use the ? operator, which simplifies code significantly
            let _: Result<(), EndOfInput> = (|| {
                // Read the header of the packet
                let header = reader.read_bytes(2)?;

                todo!();

                Ok(())
            })();
        }
    });
}