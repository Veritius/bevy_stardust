use std::{sync::Mutex, time::Instant};
use bevy::{prelude::*, utils::HashMap};
use bevy_stardust::{connections::{PeerAddress, PeerRtt}, diagnostics::DropPackets, prelude::*};
use bytes::BytesMut;
use connections::sending::stardust_transmit_many;
use datagrams::{Datagram, DatagramDesequencer, DatagramHeader, DatagramPurpose, DatagramSequencer};
use endpoints::perform_transmit;
use quinn_proto::{Connection, Dir, Event as AppEvent, SendDatagramError, StreamEvent, StreamId, VarInt};
use streams::{Recv, ResetCode, Send, SendInit, StardustRecvError, StreamReadError, StreamReader, StreamWriter};
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
            let mut connection = unsafe { connections.get_unchecked(*entity) }.unwrap();

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
    channels: Channels,
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
        let channel_map = &mut connection.channels;
        let senders = &mut connection.senders;
        let pending = &mut connection.pending;
        let desequencers = &mut connection.desequencers;

        // Poll as many events as possible from the handler
        while let Some(event) = inner.poll() { match event {
            AppEvent::Connected => {
                // Log this to the console
                info!("Connection {entity} finished handshake");

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
                info!("Connection {entity} lost: {reason}");

                // Set their lifestage to Closed.
                if let Some(ref mut lifestage) = lifestage {
                    *lifestage.as_mut() = PeerLifestage::Closed;
                }

                // Fetch the endpoint component
                let mut endpoints = endpoints.lock().unwrap();
                let (_, mut endpoint) = endpoints.get_mut(connection.owner).unwrap();

                // Remove the entity id from the map
                endpoint.entities.remove(&connection.handle);

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

                    match stream_read(
                        &config,
                        id,
                        inner,
                        readers,
                        &mut incoming,
                        pending,
                    ) {
                        Ok(_) => { /* Do nothing */ },

                        Err(err) => {
                            trace!(stream=?id, "Error while reading stream: {err:?}");

                            match err {
                                ReadFnError::Stream(StreamReadError::Closed) => {},
                                ReadFnError::Stream(StreamReadError::Reset(_code)) => {},
                                ReadFnError::Stardust(StardustRecvError::ExceededLimit) => { let _ = inner.recv_stream(id).stop(ResetCode::Violation.into()); },
                            };

                            readers.remove(&id);
                        },
                    }
                },

                StreamEvent::Readable { id } => match stream_read(
                    &config,
                    id,
                    inner,
                    readers,
                    &mut incoming,
                    pending,
                ) {
                    Ok(_) => { /* Do nothing */ },

                    Err(err) => {
                        trace!(stream=?id, "Error while reading stream: {err:?}");

                        match err {
                            ReadFnError::Stream(StreamReadError::Closed) => {},
                            ReadFnError::Stream(StreamReadError::Reset(_code)) => {},
                            ReadFnError::Stardust(StardustRecvError::ExceededLimit) => { let _ = inner.recv_stream(id).stop(ResetCode::Violation.into()); },
                        };

                        readers.remove(&id);
                    },
                },

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

                        (streams::StreamWriteOutcome::Error(_), _) => {
                            trace!("Stream reset due to error: ");
                            let _ = stream.reset(ResetCode::Unspecified.into());
                            senders.remove(&id);
                        },

                        _ => {},
                    }
                },

                StreamEvent::Finished { id } => {
                    readers.remove(&id);
                },

                StreamEvent::Stopped { id, error_code } => {
                    let code: ResetCode = (error_code.into_inner() as u32).into();
                    trace!("Remote peer stopped stream: {code}");

                    if let Some(sender) = senders.get(&id) {
                        // Remove from the channel map
                        if let Some(channel) = sender.channel() {
                            channel_map.remove(&channel);
                        }

                        // Remove the sender
                        senders.remove(&id);
                    }
                },

                // We don't care about this
                StreamEvent::Available { dir: _ } => {},
            },

            // Receive as many datagrams as possible
            AppEvent::DatagramReceived => {
                let mut datagrams = inner.datagrams();
                while let Some(mut datagram) = datagrams.recv() {
                    // Decode the datagram header and related stuff
                    let datagram = match Datagram::decode(&mut datagram) {
                        Ok(datagram) => datagram,
                        Err(err) => {
                            trace!("Error while decoding datagram: {err:?}");
                            continue;
                        },
                    };

                    match datagram.header.purpose {
                        DatagramPurpose::StardustUnordered { channel } => {
                            let channel = ChannelId::from(channel);

                            // Check the channel exists
                            if !channels.exists(channel) {
                                trace!(?channel, "Datagram had nonexistent channel id");
                            }

                            // Construct message wrapper type
                            let message = ChannelMessage {
                                channel,
                                payload: datagram.payload.clone().into(),
                            };

                            match incoming {
                                Some(ref mut queue) => queue.push_one(message),
                                None => pending.push(message),
                            }
                        },

                        DatagramPurpose::StardustSequenced { channel, sequence } => {
                            let channel = ChannelId::from(channel);

                            // Check the channel exists
                            if !channels.exists(channel) {
                                trace!(?channel, "Datagram had nonexistent channel id");
                            }

                            // Get the desequencer value
                            let desequencer = desequencers
                                .entry(channel)
                                .or_insert_with(DatagramDesequencer::new);

                            // Store in the desequencer
                            if desequencer.newer(sequence) {
                                // Construct message wrapper type
                                let message = ChannelMessage {
                                    channel,
                                    payload: datagram.payload.clone().into(),
                                };

                                match incoming {
                                    Some(ref mut queue) => queue.push_one(message),
                                    None => pending.push(message),
                                }
                            }
                        },
                    }
                }
            },

            AppEvent::DatagramsUnblocked => {},

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
    ) -> Result<(), ReadFnError> {
        let mut stream = inner.recv_stream(id);
        let recv = readers.get_mut(&id).unwrap().as_mut();

        match stream.read(true) {
            Ok(mut chunks) => {
                recv.read_from(&mut chunks).map_err(|e| ReadFnError::Stream(e))?;
                let _ = chunks.finalize();
            },

            Err(err) => return Err(match err {
                quinn_proto::ReadableError::ClosedStream => ReadFnError::Stream(StreamReadError::Closed),
                quinn_proto::ReadableError::IllegalOrderedRead => unimplemented!(),
            }),
        };

        match recv.poll(&config) {
            streams::RecvOutput::Nothing => {},

            streams::RecvOutput::Stardust(recv) => {
                let channel: ChannelId = recv.channel().into();

                for item in recv {
                    match item {
                        Ok(payload) => {
                            let message = ChannelMessage { channel, payload };

                            // TODO: Improve this
                            match incoming {
                                Some(ref mut queue) => queue.push_one(message),
                                None => pending.push(message),
                            }
                        },

                        Err(err) => return Err(ReadFnError::Stardust(err)),
                    }
                }
            },
        }

        return Ok(())
    }

    #[derive(Debug)]
    enum ReadFnError {
        Stream(StreamReadError),
        Stardust(StardustRecvError),
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

        // Storage for datagrams with an attached priority value
        let mut queued_datagrams: Vec<(u32, Datagram)> = Vec::new();

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

            // Save ourselves some work
            if messages.len() == 0 { continue }

            stardust_transmit_many(
                config,
                connection,
                &mut queued_datagrams,
                channel,
                messages
            );
        }

        // Empty the datagram queue
        if queued_datagrams.len() > 0 {
            // Logging stuff
            let span = trace_span!("Queueing datagrams");
            span.record("total", queued_datagrams.len());
            let _entered = span.entered();

            // Sort datagrams by priority
            trace_span!("Sorting datagrams").in_scope(|| {
                queued_datagrams.sort_unstable_by_key(|(k, _)| *k);
            });

            let max_size = connection.inner.datagrams().max_size().unwrap();

            // Send as many datagrams as possible
            let mut datagrams = connection.inner.datagrams();
            let mut sends: u32 = 0;
            for (_, datagram) in queued_datagrams.drain(..) {
                // Length estimate
                let len = datagram.size();

                if len > max_size {

                }

                // Create the payload thingy
                let mut payload = BytesMut::with_capacity(len);
                datagram.encode(&mut payload).unwrap();
                let payload = payload.freeze();

                // Send datagrams
                match datagrams.send(payload, false) {
                    Ok(_) => {
                        sends += 1;
                    },

                    Err(SendDatagramError::Blocked(_)) => break,

                    Err(SendDatagramError::UnsupportedByPeer) => todo!(),

                    Err(SendDatagramError::Disabled) => unimplemented!(),
                    Err(SendDatagramError::TooLarge) => unimplemented!(),
                }
            }

            // Record somes statistics
            _entered.record("sends", sends);
        }
    });
}

pub(crate) fn connection_datagram_send_system(
    config: Res<QuicConfig>,
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    connections: Query<(&mut QuicConnection, Option<&DropPackets>)>,
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

        // Random chance state for some stuff
        let mut rng = fastrand::Rng::new();

        // Iterate over all connections associated with this endpoint
        let entities = endpoint.entities.iter();
        for (handle, entity) in entities {
            // Logging stuff
            let trace_span = trace_span!("Polling connection", connection=?entity, handle=?handle);
            let _entered = trace_span.entered();

            // SAFETY: Endpoints will only access the connections they have created
            let (mut connection, reduction) = unsafe { connections.get_unchecked(*entity) }.unwrap();

            // Handle timeouts
            connection.inner.handle_timeout(Instant::now());

            // Repeatedly poll transmit until the connection no longer wants to send any more packets
            let mut send_count: u32 = 0;
            while let Some(transmit) = connection.inner.poll_transmit(Instant::now(), 1, &mut buf) {
                // Whether or not we simply 'forget' to send the packet
                // TODO: Don't check if reduction is Some every time we need to transmit
                let send = !(reduction.is_some_and(|v| rng.f32() < v.0));
                if send { perform_transmit(socket, &buf, transmit); }

                send_count += 1;
                buf.clear(); // Clear the buffer
            }

            // Record the amount of packets we've sent
            // This includes intentionally dropped packets
            _entered.record("sends", send_count);
        }
    });
}