use std::{collections::HashMap, net::SocketAddr, time::{Duration, Instant}};
use bevy_ecs::{entity::Entities, prelude::*};
use bytes::{Bytes, BytesMut};
use untrusted::*;
use crate::{appdata::{AppNetVersionWrapper, NetworkVersionData, BANNED_MINOR_VERSIONS, TRANSPORT_VERSION_DATA}, connection::{handshake::{packets::{ClientHelloPacket, ClosingPacket, HandshakePacket, HandshakePacketHeader, HandshakeParsingResponse}, HandshakeState}, reliability::{ReliabilityData, ReliablePacketHeader}, Connection, PotentialNewPeer}, endpoint::ConnectionOwnershipToken, packet::{IncomingPacket, OutgoingPacket, PacketQueue}, ConnectionDirection, ConnectionState, Endpoint};
use super::{codes::HandshakeResponseCode, packets::{ClientFinalisePacket, ServerHelloPacket}, HandshakeFailureReason};
use super::Handshaking;

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);
const RESEND_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn handshake_polling_system(
    appdata: Res<AppNetVersionWrapper>,
    commands: ParallelCommands,
    mut connections: Query<(Entity, &mut Connection, &mut Handshaking)>,
) {
    // Iterate connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut handshake)| {
        'outer: { match handshake.state {
            // Sending ClientHelloPackets to the remote peer and waiting for a ServerHelloPacket
            HandshakeState::ClientHello => {
                // Read any incoming packets
                while let Some(packet) = connection.packet_queue.pop_incoming() {
                    let mut reader = Reader::new(Input::from(&packet.payload));

                    // Try to parse the packet as a ServerHelloPacket, the next packet in the sequence
                    let packet = match ServerHelloPacket::from_reader(&mut reader) {
                        HandshakeParsingResponse::Continue(val) => val,
                        HandshakeParsingResponse::WeRejected(code) => {
                            // Set handshake state to failed
                            handshake.state = HandshakeFailureReason::WeRejected(code).into();

                            // Check if we ought to send a response packet
                            if !code.should_respond_on_rejection() { break; }

                            // Send a packet informing them of our denial
                            send_close_packet(&mut connection.packet_queue, &mut handshake.reliability, code);

                            // We're done
                            break 'outer;
                        },
                        HandshakeParsingResponse::TheyRejected(code) => {
                            // Set handshake state to failed
                            handshake.state = HandshakeFailureReason::TheyRejected(code).into();

                            // We're done
                            break 'outer;
                        },
                    };

                    // Update reliability
                    handshake.reliability.remote_sequence = packet.header.sequence;
                    let _ = handshake.reliability.ack(ReliablePacketHeader {
                        sequence: packet.header.sequence,
                        ack: packet.reliability_ack,
                        ack_bitfield: rel_bitfield_16_to_128(packet.reliability_bits),
                    }, 2);

                    // Check transport and application versions
                    for (us, them, banlist, is_app) in [
                        (&TRANSPORT_VERSION_DATA, &packet.transport, BANNED_MINOR_VERSIONS, false),
                        (&appdata.0.into_version(), &packet.application, appdata.0.banlist, true),
                    ] {
                        // Check the transport version
                        match check_identity_match(us, them, banlist, is_app) {
                            Ok(_) => {},
                            Err(code) => {
                                // Set handshake state to failed
                                handshake.state = HandshakeFailureReason::WeRejected(code).into();

                                // Send a packet informing them of our denial
                                send_close_packet(&mut connection.packet_queue, &mut handshake.reliability, code);

                                // We're done
                                break 'outer;
                            }
                        }
                    }

                    // Respond with a ClientFinalisePacket
                    let r_header = handshake.reliability.header();
                    let mut buf = BytesMut::with_capacity(6);
                    ClientFinalisePacket {
                        header: HandshakePacketHeader { sequence: r_header.sequence },
                        reliability_ack: r_header.ack,
                        reliability_bits: rel_bitfield_128_to_16(handshake.reliability.sequence_memory),
                    }.write_bytes(&mut buf);
                    connection.packet_queue.push_outgoing(OutgoingPacket::from(buf.freeze()));

                    // Mark as finalised
                    handshake.state = HandshakeState::Finished;
                }
            }

            // Sending ServerHelloPackets to the remote peer and waiting for a ClientFinalisePacket
            HandshakeState::ServerHello => todo!(),

            // Do nothing, other systems handle this
            HandshakeState::Finished | HandshakeState::Failed(_) => {}
        }
    }});
}

fn send_close_packet(
    packet_queue: &mut PacketQueue,
    reliability: &mut ReliabilityData,
    reason: HandshakeResponseCode,
) {
    // Send a packet informing them of our denial
    let r_header = reliability.header();
    packet_queue.push_outgoing(OutgoingPacket {
        payload: closing_packet(&ClosingPacket {
            header: HandshakePacketHeader { sequence: r_header.sequence },
            reason,
            additional: None,
        }),
        messages: 0,
    });
}

fn rel_bitfield_16_to_128(bitfield: u16) -> u128 {
    let a = bitfield.to_be_bytes();
    let mut b = [0u8; 16];
    b[0] = a[0]; b[1] = a[1];
    u128::from_be_bytes(b)
}

fn rel_bitfield_128_to_16(bitfield: u128) -> u16 {
    let a = bitfield.to_be_bytes();
    u16::from_be_bytes([a[0], a[1]])
}

pub(crate) fn potential_new_peers_system(
    mut events: EventReader<PotentialNewPeer>,
    appdata: Res<AppNetVersionWrapper>,
    entities: &Entities,
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
) {
    use untrusted::*;

    let mut pending: HashMap<SocketAddr, Box<(Entity, Connection, Handshaking)>> = HashMap::new();
    let mut ev_iter = events.read();
    'outer: while let Some(event) = ev_iter.next() {
        // There is the potential of bunched-up packet arrivals, so we push them to the queue just in case.
        if let Some(item) = pending.get_mut(&event.address) {
            item.1.packet_queue.push_incoming(IncomingPacket { payload: event.payload.clone() });
            continue;
        }

        // Useful things we'll be using
        let mut reader = Reader::new(Input::from(&event.payload));
        let mut endpoint = match endpoints.get_mut(event.endpoint) {
            Ok(val) => val,
            Err(_) => { continue; },
        };

        // Try to parse the UDP packet as a ClientHelloPacket struct
        let packet = match ClientHelloPacket::from_reader(&mut reader) {
            HandshakeParsingResponse::Continue(val) => val,
            HandshakeParsingResponse::WeRejected(code) => {
                // Log the disconnect
                tracing::debug!("Received and rejected connection attempt from {}: {code}",
                    event.address);

                // Check if the failure code ought to be sent to them
                // This somewhat avoids sending a packet to a peer that won't understand it
                if !code.should_respond_on_rejection() { continue }

                // Push response packet to queue
                endpoint.outgoing_pkts.push((event.address, closing_packet(&ClosingPacket {
                    header: HandshakePacketHeader { sequence: fastrand::u16(..) },
                    reason: code,
                    additional: None,
                })));

                // We're done
                continue;
            },
            HandshakeParsingResponse::TheyRejected(_) => { continue; }, // Do nothing.
        };

        // Check transport and application versions
        for (us, them, banlist, is_app) in [
            (&TRANSPORT_VERSION_DATA, &packet.transport, BANNED_MINOR_VERSIONS, false),
            (&appdata.0.into_version(), &packet.application, appdata.0.banlist, true),
        ] {
            // Check the transport version
            match check_identity_match(us, them, banlist, is_app) {
                Ok(_) => {},
                Err(code) => {
                    // Push response packet to queue
                    endpoint.outgoing_pkts.push((event.address, closing_packet(&ClosingPacket {
                        header: HandshakePacketHeader { sequence: fastrand::u16(..) },
                        reason: code,
                        additional: None,
                    })));

                    // We're done
                    continue 'outer;
                },
            }
        }

        // By this point the peer has passed all checks for their initial ClientHello packet
        // We now just have to create the relevant components and add them to the pending map

        let connection = Connection::new(
            event.endpoint,
            event.address,
            ConnectionDirection::Incoming,
        );

        // We have to construct the reliability state from scratch
        let mut reliability = ReliabilityData::new();
        reliability.remote_sequence = packet.header.sequence;
        reliability.sequence_memory |= 1u128 << 127;

        let handshake = Handshaking {
            started: Instant::now(),
            state: HandshakeState::ServerHello,
            reliability,
        };

        // Finally, add it to the map
        pending.insert(event.address, Box::new((event.endpoint, connection, handshake)));
    }

    // Add the new peers as entities
    let mut drain = pending.drain();
    while let Some((address, comp_box)) = drain.next() {
        // Get and reserve entity ids for the endpoint and connection respectively
        let ept_id = comp_box.0.clone();
        let ent_id = entities.reserve_entity();

        // Add to the endpoint connection map
        // For a small point in time, this makes it so that the endpoint map owns
        // an entity that doesn't actually exist in the world. This should be fine.
        match endpoints.get_mut(ept_id) {
            Ok(mut val) => {
                // SAFETY: The hashmap only stores one connection per address, so this is fine.
                val.add_peer(address, unsafe { ConnectionOwnershipToken::new(ent_id) })
            },
            Err(_) => { continue; },
        }

        // Defers the movement out of comp_box so we only do one memcpy instead of two
        commands.add(move |world: &mut World| {
            // I'm not sure if these semantics are necessary
            let bx = comp_box;
            let bx = *bx;

            world
                .get_or_spawn(ent_id)
                .unwrap()
                .insert((bx.1, bx.2));
        });

        // Log the new connection
        tracing::debug!("Received join request from new connection {ent_id:?} on address {address}");
    }
}

fn closing_packet(
    packet: &ClosingPacket,
) -> Bytes {
    // Allocate exactly enough space for the packet
    let mut buf = BytesMut::with_capacity(4 + match &packet.additional {
        Some(v) => v.len(),
        None => 0,
    });

    // Write the packet to the buffer
    packet.write_bytes(&mut buf);

    // Make immutable and return
    buf.freeze()
}

fn check_identity_match(
    us: &NetworkVersionData,
    them: &NetworkVersionData,
    banlist: &[u32],
    is_app: bool,
) -> Result<(), HandshakeResponseCode> {
    // Check the identity value
    if us.ident != them.ident {
        return Err(match is_app {
            true => HandshakeResponseCode::IncompatibleApplicationIdentifier,
            false => HandshakeResponseCode::IncompatibleTransportIdentifier,
        });
    }

    // Check the major version
    if us.major != them.major {
        return Err(match is_app {
            true => HandshakeResponseCode::IncompatibleApplicationMajorVersion,
            false => HandshakeResponseCode::IncompatibleTransportMajorVersion,
        });
    }

    // Check the minor version
    if banlist.contains(&them.minor) {
        return Err(match is_app {
            true => HandshakeResponseCode::IncompatibleApplicationMinorVersion,
            false => HandshakeResponseCode::IncompatibleTransportMinorVersion,
        });
    }

    Ok(())
}