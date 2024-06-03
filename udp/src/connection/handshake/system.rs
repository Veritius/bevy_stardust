use bevy::{prelude::*, utils::hashbrown::HashSet};
use bevy_stardust::prelude::*;
use bytes::BufMut;
use unbytes::Reader;
use crate::{connection::{established::Established, PotentialNewPeer}, endpoint::ConnectionOwnershipToken, plugin::PluginConfiguration, sequences::SequenceId, version::{BANNED_MINOR_VERSIONS, TRANSPORT_VERSION_DATA}};
use self::messages::*;
use super::*;

pub(in crate::connection) fn potential_incoming_system(
    config: Res<PluginConfiguration>,
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
    mut events: EventReader<PotentialNewPeer>,
) {
    let mut already_heard = HashSet::new();
    for event in events.read() {
        // Simple checks to verify validity of message
        // Prevents bunched-up handshake packets from adding multiple connection entities
        // If multiple connection entities are added, UB will occur due to data races
        if event.payload.len() < 34 { continue }
        if already_heard.contains(&event.address) { continue; }
        already_heard.insert(event.address);

        // Parse their message. We can unwrap since we checked length before.
        let mut reader = Reader::new(event.payload.clone());
        let seq_idt: SequenceId = reader.read_u16().unwrap().into();
        let hello = InitiatorHello::recv(&mut reader).unwrap();

        // Get the endpoint that sent this event
        let mut endpoint = endpoints.get_mut(event.endpoint).unwrap();

        // Check if their version is valid, if not, reject them
        if let Err(code) = version_pair_check(hello.tpt_ver, hello.app_ver, &config) {
            let mut buf = Vec::with_capacity(2);
            Rejection { code, message: Bytes::new() }.send(&mut buf).unwrap();
            endpoint.outgoing_pkts.push((event.address, Bytes::from(buf)));
            continue;
        }

        // Set up reliability
        let mut reliability = ReliabilityState::new();
        reliability.remote_sequence = seq_idt;

        // Add the connection entity
        let id = commands.spawn((
            Connection::new(event.endpoint, event.address),
            Handshaking {
                state: HandshakeState::Hello,
                started: Instant::now(),
                last_sent: None,
                scflag: true,
                direction: Direction::Listener,
                reliability,
            },
            NetworkPeer::new(),
            PeerSendBudget::new(0),
            NetworkPeerLifestage::Handshaking,
            NetworkPeerAddress(event.address),
            NetworkSecurity::Unauthenticated,
        )).id();

        // SAFETY: We can guarantee this peer is only added once here since we check earlier
        debug!("Started handshake with new peer ({:?}) with id {id:?}", event.address);
        let token = unsafe { ConnectionOwnershipToken::new(id) };
        endpoint.add_peer(event.address, token);
    }
}

pub(in crate::connection) fn handshake_polling_system(
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Handshaking)>,
) {
    // Iterate connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut handshake)| {
        // Read packets from the receive queue into the handshaking component
        while let Some(packet) = connection.recv_queue.pop_front() {
            let mut reader = Reader::new(packet);
            if reader.remaining() < 4 { continue }

            // Read the packet sequence identifier
            let seq: SequenceId = reader.read_u16().unwrap().into();

            // Special checks for different packets
            // Some packets require different handling of their sequence id
            match (handshake.state.clone(), handshake.direction) {
                (HandshakeState::Hello, Direction::Initiator) => {},
                _ => {
                    // If the packet is too old ignore it
                    if seq <= handshake.reliability.remote_sequence {
                        continue
                    }
                },
            }

            match (handshake.state.clone(), handshake.direction) {
                (HandshakeState::Hello, Direction::Initiator) => {
                    // Update the sequence value
                    handshake.reliability.remote_sequence = seq;

                    let message = match ListenerHello::recv(&mut reader) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    match message {
                        ListenerHello::Rejected(rejection) => {
                            handshake.terminate(
                                rejection.code,
                                Some(rejection.message),
                            ); break;
                        },

                        ListenerHello::Accepted {
                            tpt_ver,
                            app_ver,
                            ack_seq,
                            ack_bits,
                        } => {
                            if let Err(code) = version_pair_check(tpt_ver, app_ver, &config) {
                                handshake.terminate(code, None);
                                break;
                            }

                            let _ = handshake.reliability.ack_bits(ack_seq, ack_bits, 2);
                            handshake.reliability.advance();
                            handshake.change_state(HandshakeState::Completed);
                            break;
                        }
                    }
                },

                (HandshakeState::Hello, Direction::Listener) => {
                    let message = match InitiatorFinish::recv(&mut reader) {
                        Ok(m) => m,
                        Err(_) => continue,
                    };

                    match message {
                        InitiatorFinish::Rejected(rejection) => {
                            handshake.terminate(rejection.code, Some(rejection.message));
                            break;
                        },

                        InitiatorFinish::Accepted {
                            ack_seq,
                            ack_bits,
                        } => {
                            let _ = handshake.reliability.ack_bits(ack_seq, ack_bits, 2);
                            handshake.reliability.advance();
                            handshake.change_state(HandshakeState::Completed);
                            break;
                        },
                    }
                },

                // Do nothing.
                (HandshakeState::Completed, _) => {},
                (HandshakeState::Terminated(_), _) => {},
            }
        }

        // Timeout check
        let timed_out = handshake.started.elapsed() > HANDSHAKE_TIMEOUT;
        match (timed_out, handshake.state.clone()) {
            (true, HandshakeState::Completed) => {},
            (true, HandshakeState::Terminated(_)) => {},
            (true, _) => {
                handshake.terminate(HandshakeResponseCode::Unspecified, None);
            },
            (false, _) => {},
        }
    });
}

pub(in crate::connection) fn handshake_sending_system(
    config: Res<PluginConfiguration>,
    mut connections: Query<(Entity, &mut Connection, &mut Handshaking)>,
) {
    // Iterate connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut handshake)| {
        // Calculate whether a message needs to be sent
        let send_due = {
            let resend = match handshake.last_sent {
                Some(v) => { v.elapsed() >= RESEND_TIMEOUT } ,
                None => true,
            };

            match (handshake.scflag, resend) {
                (true, _) => true,
                (_, v) => v,
            }
        };

        // If nothing is due to send, return
        if !send_due { return }

        // Scratch space for our messaging
        let mut buf: Vec<u8> = Vec::with_capacity(64);

        // Frames are always prefixed with a sequence id
        buf.put_u16(handshake.reliability.local_sequence.into());

        match (handshake.state.clone(), handshake.direction) {
            (HandshakeState::Hello, Direction::Initiator) => {
                InitiatorHello {
                    tpt_ver: TRANSPORT_VERSION_DATA.clone(),
                    app_ver: config.application_version.clone(),
                }.send(&mut buf).unwrap();
            },

            (HandshakeState::Hello, Direction::Listener) => {
                buf.put_u16(HandshakeResponseCode::Continue as u16);
                ListenerHello::Accepted {
                    tpt_ver: TRANSPORT_VERSION_DATA.clone(),
                    app_ver: config.application_version.clone(),
                    ack_seq: handshake.reliability.remote_sequence,
                    ack_bits: handshake.reliability.ack_memory,
                }.send(&mut buf).unwrap();
            },

            (HandshakeState::Completed, Direction::Initiator) => {
                buf.put_u16(HandshakeResponseCode::Continue as u16);
                InitiatorFinish::Accepted {
                    ack_seq: handshake.reliability.remote_sequence,
                    ack_bits: handshake.reliability.ack_memory,
                }.send(&mut buf).unwrap();
            },

            (HandshakeState::Completed, Direction::Listener) => {},

            (HandshakeState::Terminated(termination), _) => {
                buf.put_u16(termination.code as u16);
                if let Some(reason) = termination.reason {
                    buf.reserve(reason.len());
                    buf.put(&reason[..]);
                }
            },
        }

        connection.send_queue.push_back(Bytes::from(buf));
        handshake.last_sent = Some(Instant::now());
    });
}

pub(in crate::connection) fn handshake_confirm_system(
    mut commands: Commands,
    mut endpoints: Query<&mut Endpoint>,
    mut connections: Query<(Entity, &Connection, &Handshaking, Option<&mut NetworkPeerLifestage>)>,
) {
    for (entity, connection, handshake, lifestage) in connections.iter_mut() {
        match &handshake.state {
            HandshakeState::Completed => {
                info!("Peer {entity:?} successfully connected");

                commands
                .entity(entity)
                .remove::<Handshaking>()
                .insert((
                    Established::new(handshake.reliability.clone()),
                    NetworkMessages::<Incoming>::new(),
                    NetworkMessages::<Outgoing>::new(),
                ));

                if let Some(mut lifestage) = lifestage {
                    *lifestage = NetworkPeerLifestage::Established;
                }
            },

            HandshakeState::Terminated(termination) => {
                debug!("Peer {entity:?} failed handshake: {}", termination.code);
                commands.entity(entity).despawn();
                
                if let Some(mut lifestage) = lifestage {
                    *lifestage = NetworkPeerLifestage::Closed;
                }

                if let Ok(mut endpoint) = endpoints.get_mut(connection.owning_endpoint) {
                    endpoint.remove_peer(entity);
                }
            },

            _ => {},
        }
    }
}

fn version_pair_check(
    tpt_ver: AppVersion,
    app_ver: AppVersion,
    config: &PluginConfiguration,
) -> Result<(), HandshakeResponseCode> {
    use HandshakeResponseCode::*;
    use crate::version::IncompatibilityReason::*;

    let tpt_chk = TRANSPORT_VERSION_DATA.compare(&tpt_ver, BANNED_MINOR_VERSIONS);
    if let Err(reason) = tpt_chk {
        return Err(match reason {
            MismatchedIdentifier => IncompatibleTransportIdentifier,
            MismatchedMajorVersion => IncompatibleApplicationMajorVersion,
            DeniedMinorVersion => IncompatibleTransportMinorVersion,
        });
    }

    let app_chk = config.application_version.compare(&app_ver, config.denied_minor_versions);
    if let Err(reason) = app_chk {
        return Err(match reason {
            MismatchedIdentifier => IncompatibleApplicationIdentifier,
            MismatchedMajorVersion => IncompatibleApplicationMajorVersion,
            DeniedMinorVersion => IncompatibleApplicationMinorVersion,
        });
    }

    return Ok(());
}