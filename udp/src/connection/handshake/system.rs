use std::time::{Instant, Duration};
use bevy_ecs::prelude::*;
use bevy_stardust::connections::peer::NetworkPeer;
use bytes::{BufMut, Bytes, BytesMut};
use untrusted::*;
use crate::{appdata::{AppNetVersionWrapper, NetworkVersionData, BANNED_MINOR_VERSIONS, TRANSPORT_VERSION_DATA}, connection::{established::Established, reliability::{ReliabilityData, ReliablePacketHeader}, PotentialNewPeer}, endpoint::ConnectionOwnershipToken, packet::OutgoingPacket, Connection, ConnectionDirection, ConnectionState, Endpoint};
use crate::utils::IntegerFromByteSlice;
use super::{codes::HandshakeErrorCode, packets::{ClientFinalisePacket, ClientHelloPacket}};
use super::{codes::{response_code_from_int, HandshakeCode}, packets::ServerHelloPacket, HandshakeFailureReason, HandshakeState, Handshaking};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(15);
const RESEND_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn handshake_polling_system(
    appdata: Res<AppNetVersionWrapper>,
    commands: ParallelCommands,
    mut connections: Query<(Entity, &mut Connection, &mut Handshaking)>,
) {
    // IDK the performance of Instant::now(), but since it's a syscall,
    // err on the side of caution and do it once at the start of the system.
    let system_start_time = Instant::now();

    // Iterate connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut handshake)| {
        // Read packets
        'ibk: { match handshake.state {
            HandshakeState::ClientHello => {
                // Check if we have any packets to receive
                if let Some(packet) = connection.packet_queue.pop_incoming() {
                    let mut reader = Reader::new(Input::from(&packet.payload));

                    // Check the packet is long enough to be useful
                    if packet.payload.len() < 2 {
                        break 'ibk; // Skip this packet
                    };

                    // Check the packet response code to see what the packet means
                    let response_code = response_code_from_int(u16::from_byte_slice(&mut reader).unwrap());
                    if let HandshakeCode::Err(error_code) = response_code {
                        handshake.state = HandshakeState::Failed(HandshakeFailureReason::TheyRejected(error_code));
                        break 'ibk; // Terminate processing
                    }

                    // Try to parse the packet as a ServerHelloPacket, the next packet expected
                    let server_hello_packet = match ServerHelloPacket::from_bytes(&mut reader) {
                        Ok(v) => v,
                        Err(_) => {
                            handshake.state = HandshakeFailureReason::BadResponse.into();
                            break 'ibk;
                        },
                    };

                    // Check the transport identity match
                    if let Err(error_code) = check_identity_match(
                        &TRANSPORT_VERSION_DATA,
                        &server_hello_packet.transport,
                        BANNED_MINOR_VERSIONS,
                        false
                    ) {
                        handshake.state = HandshakeFailureReason::WeRejected(error_code).into();
                        break 'ibk;
                    }

                    // Check the application identity match
                    if let Err(error_code) = check_identity_match(
                        &TRANSPORT_VERSION_DATA,
                        &server_hello_packet.transport,
                        appdata.0.banlist,
                        true
                    ) {
                        handshake.state = HandshakeFailureReason::WeRejected(error_code).into();
                        break 'ibk;
                    }

                    // Update reliability data
                    handshake.reliability.remote_sequence = server_hello_packet.reliability.sequence;
                    handshake.reliability.local_sequence += 1;

                    // Respond with packet
                    let mut buf = BytesMut::with_capacity(10);

                    buf.put_u16(0); // Ok marker

                    ClientFinalisePacket {
                        reliability: ReliablePacketHeader {
                            sequence: handshake.reliability.local_sequence,
                            ack: handshake.reliability.remote_sequence,
                            ack_bitfield: handshake.reliability.sequence_memory,
                        }
                    }.write_bytes(&mut buf);

                    connection.packet_queue.push_outgoing(OutgoingPacket {
                        payload: buf.freeze(),
                        messages: 0,
                    });

                    // Mark as finished and break
                    handshake.state = HandshakeState::Finished;
                    break 'ibk;
                }

                // Check if we need to send a message
                if connection.timings.last_sent.is_none() || system_start_time.saturating_duration_since(connection.timings.last_sent.unwrap()) > RESEND_TIMEOUT {
                    let mut buf = BytesMut::with_capacity(34);

                    ClientHelloPacket {
                        transport: TRANSPORT_VERSION_DATA.clone(),
                        application: appdata.0.into_version(),
                        sequence_identifier: handshake.reliability.local_sequence,
                    }.write_bytes(&mut buf);

                    connection.packet_queue.push_outgoing(OutgoingPacket {
                        payload: buf.freeze(),
                        messages: 0,
                    });
                }
            },

            HandshakeState::ServerHello => {
                // Check if we have any packets to receive
                if let Some(packet) = connection.packet_queue.pop_incoming() {
                    let mut reader = Reader::new(Input::from(&packet.payload));

                    // Check the packet is long enough to be useful
                    if packet.payload.len() < 2 {
                        break 'ibk; // Skip this packet
                    }
                    
                    // Check the packet response code to see what the packet means
                    let response_code = response_code_from_int(u16::from_byte_slice(&mut reader).unwrap());
                    if let HandshakeCode::Err(error_code) = response_code {
                        handshake.state = HandshakeState::Failed(HandshakeFailureReason::TheyRejected(error_code));
                        break 'ibk; // Terminate processing
                    }

                    // Try to parse the packet as a ClientFinalisePacket, the next packet expected
                    let client_finalise_packet = match ClientFinalisePacket::from_bytes(&mut reader) {
                        Ok(v) => v,
                        Err(_) => {
                            handshake.state = HandshakeFailureReason::BadResponse.into();
                            break 'ibk;
                        },
                    };

                    // Update reliability data
                    handshake.reliability.remote_sequence = client_finalise_packet.reliability.sequence;

                    // Mark as finished and break
                    handshake.state = HandshakeState::Finished;
                    break 'ibk;
                }

                // Check if we need to send a message
                if connection.timings.last_sent.is_none() || system_start_time.saturating_duration_since(connection.timings.last_sent.unwrap()) > RESEND_TIMEOUT  {
                    let mut buf = BytesMut::with_capacity(42);

                    buf.put_u16(0); // Ok marker

                    ServerHelloPacket {
                        transport: TRANSPORT_VERSION_DATA.clone(),
                        application: appdata.0.into_version(),
                        reliability: ReliablePacketHeader {
                            sequence: handshake.reliability.local_sequence,
                            ack: handshake.reliability.remote_sequence,
                            ack_bitfield: handshake.reliability.sequence_memory,
                        },
                    }.write_bytes(&mut buf);

                    connection.packet_queue.push_outgoing(OutgoingPacket {
                        payload: buf.freeze(),
                        messages: 0,
                    });
                }
            },

            // We don't do anything in these states
            HandshakeState::Finished | HandshakeState::Failed(_) => {},
        }}

        // Change state based on timings
        if !handshake.state.is_end() {
            if handshake.started.saturating_duration_since(system_start_time) > HANDSHAKE_TIMEOUT {
                handshake.state = HandshakeState::Failed(HandshakeFailureReason::TimedOut);
            }
        }

        // Apply commands based on state
        match &handshake.state {
            HandshakeState::Finished => {
                // Log handshake success and update entity
                tracing::info!("Connection handshake to {} succeeded",
                    connection.remote_address());

                // Remove Handshaking component and add Established component
                commands.command_scope(|mut commands| {
                    commands.entity(entity)
                        .remove::<Handshaking>()
                        .insert(Established::new())
                        .insert(NetworkPeer::new());
                });
            },
            HandshakeState::Failed(reason) => {
                // Log handshake failure and reason
                tracing::info!("Connection handshake to {} failed: {reason}",
                    connection.remote_address());

                // Mark the connection as closed so it gets despawned
                connection.connection_state = ConnectionState::Closed;
            },
            _ => {},
        }
    });
}

pub(crate) fn potential_new_peers_system(
    mut events: EventReader<PotentialNewPeer>,
    appdata: Res<AppNetVersionWrapper>,
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
) {
    let mut ev_iter = events.read();
    while let Some(event) = ev_iter.next() {
        let mut reader = Reader::new(Input::from(&event.payload));

        // Turn bytes into a packet
        let pkt = match ClientHelloPacket::from_bytes(&mut reader) {
            Ok(pkt) => pkt,
            Err(_) => { continue }, // Not long enough, ignore it.
        };

        // Get the endpoint component
        let mut endpoint = match endpoints.get_mut(event.ep_origin) {
            Ok(v) => v,
            Err(_) => { continue },
        };

        // Check the transport version
        match check_identity_match(
            &TRANSPORT_VERSION_DATA,
            &pkt.transport,
            &BANNED_MINOR_VERSIONS,
            false
        ) {
            Ok(_) => {},
            Err(err) => {
                // Inform them that they can't connect
                endpoint.outgoing_pkts.push((
                    event.address,
                    Bytes::copy_from_slice(&(err as u16).to_be_bytes()),
                ));

                tracing::debug!("Rejected handshake from {} due to: {err}", event.address);
            },
        }

        // Check the application version
        match check_identity_match(
            &NetworkVersionData {
                ident: appdata.0.ident,
                major: appdata.0.major,
                minor: appdata.0.minor,
            },
            &pkt.application,
            appdata.0.banlist,
            true
        ) {
            Ok(_) => {},
            Err(err) => {
                // Inform them that they can't connect
                endpoint.outgoing_pkts.push((
                    event.address,
                    Bytes::copy_from_slice(&(err as u16).to_be_bytes()),
                ));

                tracing::debug!("Rejected handshake from {} due to: {err}", event.address);
            },
        }

        // They've succeeded our checks

        let mut connection = Connection::new(
            event.ep_origin,
            event.address,
            ConnectionDirection::Incoming
        );

        let mut reliability = ReliabilityData::new();
        reliability.remote_sequence = pkt.sequence_identifier;

        let mut buf = BytesMut::with_capacity(40);
        buf.put_u16(0);
        ServerHelloPacket {
            transport: TRANSPORT_VERSION_DATA.clone(),
            application: NetworkVersionData {
                ident: appdata.0.ident,
                major: appdata.0.major,
                minor: appdata.0.minor,
            },
            reliability: ReliablePacketHeader {
                sequence: reliability.local_sequence,
                ack: reliability.remote_sequence,
                ack_bitfield: reliability.sequence_memory,
            },
        }.write_bytes(&mut buf);

        connection.packet_queue.push_outgoing(OutgoingPacket { 
            payload: buf.freeze(),
            messages: 0,
        });

        let id = commands.spawn((
            connection,
            Handshaking {
                started: Instant::now(),
                state: HandshakeState::ServerHello,
                reliability,
            },
        )).id();

        // SAFETY: This is fine, since this is the only system that spawns incoming handshaking peers.
        endpoint.add_peer(event.address, unsafe { ConnectionOwnershipToken::new(id) });

        tracing::debug!("Responded to new peer on address {}, now with id {id:?}", event.address);
    }
}

fn check_identity_match(
    us: &NetworkVersionData,
    them: &NetworkVersionData,
    banlist: &[u32],
    is_app: bool,
) -> Result<(), HandshakeErrorCode> {
    // Check the identity value
    if us.ident != them.ident {
        return Err(match is_app {
            true => HandshakeErrorCode::IncompatibleApplicationIdentifier,
            false => HandshakeErrorCode::IncompatibleTransportIdentifier,
        });
    }

    // Check the major version
    if us.major != them.major {
        return Err(match is_app {
            true => HandshakeErrorCode::IncompatibleApplicationMajorVersion,
            false => HandshakeErrorCode::IncompatibleTransportMajorVersion,
        });
    }

    // Check the minor version
    if banlist.contains(&them.minor) {
        return Err(match is_app {
            true => HandshakeErrorCode::IncompatibleApplicationMinorVersion,
            false => HandshakeErrorCode::IncompatibleTransportMinorVersion,
        });
    }

    Ok(())
}