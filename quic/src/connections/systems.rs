use std::{sync::Mutex, time::Instant};
use bevy::{prelude::*, utils::HashMap};
use bevy_stardust::{connections::{PeerAddress, PeerRtt}, prelude::*};
use endpoints::perform_transmit;
use quinn_proto::{Connection, Dir, Event as AppEvent, StreamEvent, StreamId, VarInt};
use streams::{Recv, Send, SendInit, StreamReader, StreamWriter};
use crate::*;

pub(crate) fn connection_update_rtt_system(
    mut query: Query<(&QuicConnection, &mut PeerRtt)>,
) {
    query.par_iter_mut().for_each(|(conn, mut rtt)| {
        let lrtt = conn.inner.rtt();
        if lrtt == rtt.0 { return };
        rtt.0 = lrtt;
    });
}

pub(crate) fn connection_endpoint_events_system(
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    connections: Query<&mut QuicConnection>,
) {
    endpoints.par_iter_mut().for_each(|(entity, mut endpoint)| {
        // Logging stuff
        let trace_span = trace_span!("Event interchange for endpoint", endpoint=?entity);
        let _entered = trace_span.entered();

        // Some stuff related to the endpoint
        let endpoint = endpoint.as_mut();

        // Iterate over all connections associated with this endpoint
        let entities = endpoint.entities.iter();
        for (handle, entity) in entities {
            // Logging stuff
            let trace_span = trace_span!("Event interchange for connection", connection=?entity, handle=?handle);
            let _entered = trace_span.entered();

            // SAFETY: Endpoints will only access the connections they have created
            let query_item = unsafe { connections.get_unchecked(*entity) };
            let mut connection = match query_item {
                Ok(connection) => connection,
                Err(err) => todo!(),
            };

            // Extract all endpoint events and give them to the endpoint
            while let Some(event) = connection.inner.poll_endpoint_events() {
                if let Some(event) = endpoint.inner.handle_event(*handle, event) {
                    connection.inner.handle_event(event);
                }
            }
        }
    });
}

pub(crate) fn connection_event_handler_system(
    config: Res<QuicConfig>,
    commands: ParallelCommands,
    mut connections: Query<(Entity, &mut QuicConnection, Option<&mut PeerLifestage>, Option<&mut PeerMessages<Incoming>>)>,
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    mut dc_events: EventWriter<PeerDisconnectedEvent>,
) {
    // Wrap the commands queue, query and eventwriter in a mutex so we can use them in parallel
    // Accesses (should be) infrequent enough that this is fine.
    let endpoints = Mutex::new(&mut endpoints);
    let dc_events = Mutex::new(&mut dc_events);

    // Iterate all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, mut lifestage, mut incoming)| {
        // Logging stuff
        let trace_span = trace_span!("Handling connection events", connection=?entity);
        let _entered = trace_span.entered();

        // Split borrows
        let connection = connection.as_mut();
        let inner = connection.inner.as_mut();
        let readers = &mut connection.readers;
        let senders = &mut connection.senders;
        let pending = &mut connection.pending;

        // Poll as many events as possible from the handler
        while let Some(event) = inner.poll() { match event {
            AppEvent::Connected => {
                // Log this to the console
                info!("Connection {entity:?} finished handshake");

                // Set their lifestage to Established.
                if let Some(ref mut lifestage) = lifestage {
                    *lifestage.as_mut() = PeerLifestage::Established;
                }

                // Add the necessary components
                commands.command_scope(|mut commands| {
                    commands.entity(entity).insert((
                        Peer::new(),
                        PeerRtt(inner.rtt()),
                        PeerAddress(inner.remote_address()),
                        PeerMessages::<Incoming>::new(),
                        PeerMessages::<Outgoing>::new(),
                    ));
                });
            },

            AppEvent::ConnectionLost { reason } => {
                // Log this to the console
                info!("Connection {entity:?} lost: {reason}");

                // Set their lifestage to Closed.
                if let Some(ref mut lifestage) = lifestage {
                    *lifestage.as_mut() = PeerLifestage::Closed;
                }

                // Fetch the endpoint component
                let mut endpoints = endpoints.lock().unwrap();
                let (_, mut endpoint) = match endpoints.get_mut(connection.owner) {
                    Ok(endpoint) => endpoint,
                    Err(_) => todo!(),
                };

                // Remove the entity id from the map
                match endpoint.entities.remove(&connection.handle) {
                    Some(_) => {},
                    None => todo!(),
                }

                // Manually drop the lock to release it early
                // This probably doesn't make any difference but whatever
                drop(endpoints);

                // Queue the entity to be despawned
                commands.command_scope(|mut commands| {
                    commands.entity(entity).despawn()
                });

                // Notify other systems of the disconnection
                dc_events.lock().unwrap().send(PeerDisconnectedEvent {
                    peer: entity,
                    reason: DisconnectReason::Unspecified, // TODO
                    comment: None,
                });
            },

            AppEvent::Stream(event) => match event {
                StreamEvent::Opened { dir } => {
                    let id = inner.streams().accept(dir).unwrap();
                    readers.insert(id, Box::new(Recv::new()));

                    stream_read(
                        &config,
                        id,
                        inner,
                        readers,
                        &mut incoming,
                        pending,
                    );
                },

                StreamEvent::Readable { id } => stream_read(
                    &config,
                    id, inner,
                    readers,
                    &mut incoming,
                    pending,
                ),

                StreamEvent::Writable { id } => {
                    let mut stream = inner.send_stream(id);
                    let send = senders.get_mut(&id).unwrap().as_mut();

                    match (send.write(&mut stream), send.transient()) {
                        // Transient senders are removed when done sending
                        (streams::StreamWriteOutcome::Complete, true) => {
                            trace!("Closed transient stream {id}");
                            let _ = stream.finish();
                            senders.remove(&id);
                        },

                        (streams::StreamWriteOutcome::Error(_), _) => todo!(),
                        _ => {},
                    }
                },

                StreamEvent::Finished { id } => {
                    senders.remove(&id);

                    // TODO: Remove from channel map
                },

                StreamEvent::Stopped { id, error_code: _ } => todo!(),

                // We don't care about this
                StreamEvent::Available { dir: _ } => {},
            },

            AppEvent::DatagramReceived => todo!(),
            AppEvent::DatagramsUnblocked => todo!(),

            // We don't care about this one.
            AppEvent::HandshakeDataReady => {},
        }}
    });

    fn stream_read<'a>(
        config: &QuicConfig,
        id: StreamId,
        inner: &mut Connection,
        readers: &mut HashMap<StreamId, Box<Recv>>,
        incoming: &mut Option<Mut<'a, PeerMessages<Incoming>>>,
        pending: &mut Vec<ChannelMessage>,
    ) {
        let mut stream = inner.recv_stream(id);
        let recv = readers.get_mut(&id).unwrap().as_mut();

        match stream.read(true) {
            Ok(mut chunks) => {
                if let Err(err) = recv.read_from(&mut chunks) {
                    todo!()
                }

                let _ = chunks.finalize();
            },

            Err(err) => todo!(),
        };

        match recv.poll(&config) {
            streams::RecvOutput::Nothing => {},

            streams::RecvOutput::Stardust(recv) => {
                let channel: ChannelId = recv.channel().into();
                let iter = recv.map(|b| Message::from_bytes(b));

                match incoming {
                    Some(ref mut queue) => queue.push_channel(channel, iter),
                    None => pending.extend(iter.map(|payload| ChannelMessage { channel, payload })),
                };
            },
        }
    }
}

pub(crate) fn connection_dump_pending_system(
    mut connections: Query<(&mut QuicConnection, &mut PeerMessages<Incoming>)>,
) {
    connections.par_iter_mut().for_each(|(mut connection, mut incoming)| {
        if connection.pending.len() == 0 { return; }
        incoming.push_many(connection.pending.drain(..));
        connection.pending.shrink_to(0);
    });
}

pub(crate) fn connection_disconnect_system(
    mut dc_requests: EventReader<DisconnectPeerEvent>,
    mut dc_occurred: EventWriter<PeerDisconnectingEvent>,
    mut connections: Query<&mut QuicConnection>,
) {
    for req in dc_requests.read() {
        if let Ok(mut connection) = connections.get_mut(req.peer) {
            connection.inner.close(
                Instant::now(),
                VarInt::from_u32(0),
                Bytes::new(),
            );

            dc_occurred.send(PeerDisconnectingEvent { peer: req.peer });
        }
    }
}

pub(crate) fn connection_message_sender_system(
    channels: Channels,
    mut connections: Query<(Entity, &mut QuicConnection, &PeerMessages<Outgoing>)>,
) {
    // Iterate all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, outgoing)| {
        // Logging stuff
        let trace_span = trace_span!("Queuing message sends", connection=?entity);
        let _entered = trace_span.entered();

        // Split borrows
        let connection = connection.as_mut();
        let inner = connection.inner.as_mut();
        let channel_map = &mut connection.channels;
        let senders = &mut connection.senders;

        // Iterate over all channels
        for (channel, messages) in outgoing.iter() {
            // Get the channel data
            let config = match channels.config(channel) {
                Some(channel_data) => channel_data,
                None => {
                    error!("Tried to send a message to a channel that didn't exist ({channel:?})");
                    continue;
                },
            };

            // Different channels have different config requirements
            use ChannelConsistency::*;
            match config.consistency {
                UnreliableUnordered => {},

                UnreliableSequenced => {},

                ReliableUnordered => {
                    for message in messages {
                        // Open a new outgoing, unidirectional stream
                        let id = inner.streams().open(Dir::Uni).unwrap();
                        let mut stream = inner.send_stream(id);
                        stream.set_priority(map_priority_value(config.priority)).unwrap();
                        trace!(?channel, stream=?id, "Opened stream for reliable unordered messages");

                        // Create a new sender
                        let mut send = Send::new(SendInit::StardustTransient { channel: channel.into() });

                        // Add the message
                        send.push(message.into());

                        // Try to write as much as possible to the stream
                        match send.write(&mut stream) {
                            // The entire send buffer was written
                            streams::StreamWriteOutcome::Complete => {
                                trace!(?channel, stream=?id, "ReliableUnordered stream did full transmit and was finished");
                                stream.finish().unwrap();
                            },

                            // Only a portion of the send buffer was written
                            streams::StreamWriteOutcome::Partial(_) |
                            streams::StreamWriteOutcome::Blocked => {
                                let boxed = Box::new(send);
                                senders.insert(id, boxed);
                            },

                            streams::StreamWriteOutcome::Error(_) => todo!(),
                        }
                    }
                },

                ReliableOrdered => {
                    // Get the ID of the channel
                    let id = channel_map.entry(channel).or_insert_with(|| {
                        // Open a new outgoing, unidirectional stream
                        let id = inner.streams().open(Dir::Uni).unwrap();
                        inner.send_stream(id).set_priority(map_priority_value(config.priority)).unwrap();
                        trace!(?channel, stream=?id, "Opened stream for reliable ordered messages");
                        id
                    }).clone();

                    // Get the sender queue
                    let send = senders.entry(id).or_insert_with(|| {
                        Box::new(Send::new(SendInit::StardustPersistent { channel: channel.into() }))
                    }).as_mut();

                    // Put all messages into the sender
                    for message in messages {
                        send.push(message.into());
                    }

                    // Try to write as much as possible to the stream
                    let mut stream = inner.send_stream(id);
                    match send.write(&mut stream) {
                        streams::StreamWriteOutcome::Error(_) => todo!(),
                        _ => {},
                    }
                },

                _ => unimplemented!()
            }
        }
    });
}

pub(crate) fn connection_datagram_send_system(
    config: Res<QuicConfig>,
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    connections: Query<&mut QuicConnection>,
) {
    endpoints.par_iter_mut().for_each(|(entity, mut endpoint)| {
        // Logging stuff
        let trace_span = trace_span!("Sending packets from endpoint", endpoint=?entity);
        let _entered = trace_span.entered();

        // Some stuff related to the endpoint
        let endpoint = endpoint.as_mut();
        let socket = &mut endpoint.socket;

        // Allocate a buffer to store messages in
        let mut buf = Vec::with_capacity(config.maximum_transport_units);

        // Iterate over all connections associated with this endpoint
        let entities = endpoint.entities.iter();
        for (handle, entity) in entities {
            // Logging stuff
            let trace_span = trace_span!("Polling connection", connection=?entity, handle=?handle);
            let _entered = trace_span.entered();

            // SAFETY: Endpoints will only access the connections they have created
            let query_item = unsafe { connections.get_unchecked(*entity) };
            let mut connection = match query_item {
                Ok(connection) => connection,
                Err(err) => todo!(),
            };

            // Handle timeouts
            connection.inner.handle_timeout(Instant::now());

            // Repeatedly poll transmit until the connection no longer wants to send any more packets
            let mut send_count: u32 = 0;
            while let Some(transmit) = connection.inner.poll_transmit(Instant::now(), 1, &mut buf) {
                perform_transmit(socket, &buf, transmit);
                send_count += 1;
                buf.clear(); // Clear the buffer
            }

            // Record the amount of packets we've sent
            _entered.record("sends", send_count);
        }
    });
}

#[inline]
fn map_priority_value(priority: u32) -> i32 {
    TryInto::<i32>::try_into(priority).unwrap_or(i32::MAX)
}