use std::{collections::BTreeMap, sync::Mutex, time::{Duration, Instant}};
use anyhow::{bail, Result};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use bytes::Bytes;
use endpoints::perform_transmit;
use quinn_proto::{coding::Codec, Connection, ConnectionHandle, ConnectionStats, Dir, Event as AppEvent, FinishError, StreamEvent, StreamId, VarInt, WriteError};
use streams::{FramedWriter, StreamOpenHeader, StreamReaderSegment, StreamReader};
use crate::*;

/// A QUIC connection, attached to an endpoint.
/// 
/// # Safety
/// This component must always stay in the same [`World`] as it was created in.
/// Being put into another `World` will lead to undefined behavior.
#[derive(Component)]
pub struct QuicConnection {
    pub(crate) owner: Entity,
    pub(crate) handle: ConnectionHandle,
    pub(crate) inner: Box<Connection>,

    channel_streams: BTreeMap<ChannelId, StreamId>,
    framed_readers: BTreeMap<StreamId, StreamReader>,
    framed_writers: BTreeMap<StreamId, FramedWriter>,
}

impl QuicConnection {
    pub(crate) fn new(
        owner: Entity,
        handle: ConnectionHandle,
        inner: Box<Connection>,
    ) -> Self {
        Self {
            owner,
            handle,
            inner,

            channel_streams: BTreeMap::new(),
            framed_readers: BTreeMap::new(),
            framed_writers: BTreeMap::new(),
        }
    }

    /// Begins closing the connection.
    pub fn close(&mut self, reason: Bytes) {
        self.inner.close(
            Instant::now(),
            DisconnectCode::AppDisconnect.try_into().unwrap(),
            reason
        );
    }

    /// Returns the current best estimate of the connection's round-trip time.
    pub fn rtt(&self) -> Duration {
        self.inner.rtt()
    }

    /// Returns the full collection of statistics for the connection.
    pub fn stats(&self) -> ConnectionStats {
        self.inner.stats()
    }

    /// Closes a stream used to send Stardust messages, releasing some resources.
    /// This is useful as an optimisation for channels that are never used after a certain point.
    /// If the channel continues to be used, a new stream will be opened.
    pub fn close_stardust_stream(&mut self, channel: ChannelId) -> Result<()> {
        if let Some(stream_id) = self.channel_streams.get(&channel) {
            // Check if the framed writer has any data in it
            if let Some(writer) = self.framed_writers.get(stream_id) {
                if writer.unsent() != 0 { bail!("Stream had unsent framed messages"); }
                self.framed_writers.remove(stream_id);
            }

            // Try to close the stream
            match self.inner.send_stream(*stream_id).finish() {
                Ok(()) => {},
                Err(FinishError::ClosedStream) => {},
                Err(FinishError::Stopped(code)) => bail!("Stream was stopped by remote: {code}"),
            }

            // Remove from map
            self.channel_streams.remove(&channel);
            return Ok(())
        } else {
            // No work to do
            return Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DisconnectCode {
    Invalid,

    Unspecified,
    AppDisconnect,
    NotListening,
}

impl From<VarInt> for DisconnectCode {
    fn from(value: VarInt) -> Self {
        use DisconnectCode::*;
        match u64::from(value) {
            0 => Unspecified,
            1 => AppDisconnect,
            2 => NotListening,

            _ => Invalid,
        }
    }
}

impl TryFrom<DisconnectCode> for VarInt {
    type Error = ();

    fn try_from(value: DisconnectCode) -> Result<Self, Self::Error> {
        use DisconnectCode::*;
        return Ok(VarInt::from_u32(match value {
            // Special case: this variant can't be sent
            Invalid => { return Err(()) },

            Unspecified => 0,
            AppDisconnect => 1,
            NotListening => 2,
        }));
    }
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
    mut connections: Query<(Entity, &mut QuicConnection, Option<&mut NetworkPeerLifestage>, Option<&mut NetworkMessages<Incoming>>)>,
    mut commands: Commands,
    mut endpoints: Query<(Entity, &mut QuicEndpoint)>,
    mut dc_events: EventWriter<PeerDisconnectedEvent>,
) {
    // Wrap the commands queue, query and eventwriter in a mutex so we can use them in parallel
    // Accesses (should be) infrequent enough that this is fine.
    let commands = Mutex::new(&mut commands);
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
        let channel_streams = &mut connection.channel_streams;
        let framed_readers = &mut connection.framed_readers;
        let framed_writers = &mut connection.framed_writers;

        // Poll as many events as possible from the handler
        while let Some(event) = inner.poll() { match event {
            AppEvent::Connected => {
                // Log this to the console
                info!("Connection {entity:?} finished handshake");

                // Set their lifestage to Established.
                if let Some(ref mut lifestage) = lifestage {
                    *lifestage.as_mut() = NetworkPeerLifestage::Established;
                }

                // Add the necessary components
                commands.lock().unwrap().entity(entity).insert((
                    NetworkPeer::new(),
                    NetworkPeerAddress(inner.remote_address()),
                    NetworkMessages::<Incoming>::new(),
                    NetworkMessages::<Outgoing>::new(),
                ));
            },

            AppEvent::ConnectionLost { reason } => {
                // Log this to the console
                info!("Connection {entity:?} lost: {reason}");

                // Set their lifestage to Closed.
                if let Some(ref mut lifestage) = lifestage {
                    *lifestage.as_mut() = NetworkPeerLifestage::Closed;
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
                commands.lock().unwrap().entity(entity).despawn();

                // Notify other systems of the disconnection
                dc_events.lock().unwrap().send(PeerDisconnectedEvent {
                    peer: entity,
                    reason: None, // TODO: give good reasons
                });
            },

            AppEvent::Stream(event) => match event {
                StreamEvent::Opened { dir } => {
                    // Accept the stream
                    let stream_id = inner.streams().accept(dir).unwrap();
                    framed_readers.insert(stream_id, StreamReader::default());
                },

                StreamEvent::Readable { id } => {
                    if let Some(reader) = framed_readers.get_mut(&id) {
                        // Read chunks from the stream into the table
                        let mut stream = inner.recv_stream(id);
                        match stream.read(true) {
                            // A chunk iterator is available
                            Ok(mut chunks) => {
                                // Try to read as many chunks as possible
                                loop { match chunks.next(1024) {
                                    // A chunk of data is readable
                                    Ok(Some(chunk)) => {
                                        // Push to the reader
                                        reader.push(chunk.bytes);
                                    },

                                    // We've run out of chunks to read
                                    Ok(None) => break,

                                    // Error while reading chunks
                                    Err(_) => break,
                                }}

                                // MUST_USE: We poll sends anyway
                                let _ = chunks.finalize();
                            },

                            // Error encountered when reading stream
                            Err(error) => todo!(),
                        }

                        // Try to read any available frames
                        for item in reader.iter(config.maximum_framed_message_length) {
                            match item {
                                // A chunk of data was read
                                Ok(StreamReaderSegment::Stardust { channel, payload }) => {
                                    if let Some(ref mut incoming) = incoming {
                                        incoming.push(channel, payload);
                                    }
                                },

                                // Error while reading
                                Err(err) => todo!(),
                            }
                        }
                    }
                },

                StreamEvent::Writable { id } => {
                    // If the stream is writable, check to see if there's any messages waiting
                    if let Some(writer) = framed_writers.get_mut(&id) {
                        let mut stream = inner.send_stream(id);
                        match writer.write(&mut stream) {
                            Ok(_) => {},
                            Err(error) => {
                                // If it errors, dump the messages
                                debug!("Error while writing framed messages on stream {id}: {error}");
                                framed_writers.remove(&id);
                                continue;
                            },
                        }

                        // If the writer is out of frames, remove it
                        if writer.unsent() == 0 {
                            framed_writers.remove(&id);
                            continue;
                        }
                    }
                },

                StreamEvent::Finished { id } |
                StreamEvent::Stopped { id, error_code: _ } => {
                    // Stopping a stream discards all data
                    // as it is a reset, not a finish.
                    framed_writers.remove(&id);
                    framed_readers.remove(&id);

                    // Find the channel id by the stream id
                    // This is an expensive operation but it won't happen often (probably)
                    let cid = channel_streams
                        .iter()
                        .find(|(_, sid)| **sid == id)
                        .map(|(cid, _)| cid)
                        .copied();

                    // Remove the channel ID from the map
                    if let Some(cid) = cid {
                        channel_streams.remove(&cid);
                    }

                    // Log the stream removal
                    debug!("Stream {id} was stopped or finished");
                }

                // We don't care about this
                StreamEvent::Available { dir: _ } => {},
            },

            AppEvent::DatagramReceived => todo!(),
            AppEvent::DatagramsUnblocked => todo!(),

            // We don't care about this one.
            AppEvent::HandshakeDataReady => {},
        }}
    });
}

pub(crate) fn connection_message_sender_system(
    registry: Res<ChannelRegistry>,
    mut connections: Query<(Entity, &mut QuicConnection, &NetworkMessages<Outgoing>)>,
) {
    // Iterate all connections in parallel
    connections.par_iter_mut().for_each(|(entity, mut connection, outgoing)| {
        // Logging stuff
        let trace_span = trace_span!("Queuing message sends", connection=?entity);
        let _entered = trace_span.entered();

        // Tiny scratch space for some operations
        let mut scr = Vec::with_capacity(4);

        // Split borrows
        let connection = connection.as_mut();
        let inner = connection.inner.as_mut();
        let channel_streams = &mut connection.channel_streams;
        let framed_writers = &mut connection.framed_writers;

        // Iterate over all channels
        'outgoing: for (channel, messages) in outgoing.iter() {
            scr.clear();

            // Get the channel data
            let channel_data = match registry.channel_config(channel) {
                Some(channel_data) => channel_data,
                None => {
                    error!("Tried to send a message to a channel that didn't exist ({channel:?})");
                    continue;
                },
            };

            // Different channels have different config requirements
            use {ReliabilityGuarantee::*, OrderingGuarantee::*};
            match (channel_data.reliable, channel_data.ordered) {
                (Unreliable, Unordered) => todo!(),

                (Unreliable, Sequenced) => todo!(),

                (Unreliable, Ordered) => todo!(),

                (Reliable, Unordered) => todo!(),

                (Reliable, Sequenced) => todo!(),

                (Reliable, Ordered) => {
                    // Get the stream ID of this channel
                    let (stream_id, mut stream) = match channel_streams.get(&channel) {
                        Some(stream_id) => {
                            (*stream_id, inner.send_stream(*stream_id))
                        },
                        None => {
                            // Wasn't in the map, we have to open a new stream
                            let stream_id = inner.streams().open(Dir::Uni).unwrap();
                            channel_streams.insert(channel, stream_id);
                            (stream_id, inner.send_stream(stream_id))
                        },
                    };

                    // Try to get a writer from the writer map
                    match framed_writers.get_mut(&stream_id) {
                        // If there's a writer in the map, it means there are
                        // unsent messages, so we add messages to the writer's queue
                        Some(writer) => {
                            // Queue all messages in the writer
                            for payload in messages.iter().cloned() {
                                writer.queue_framed(payload);
                            }
                        },

                        // If there's no writer in the map, it means
                        // there are no messages queued, so we need a
                        // new FramedWriter instance
                        None => {
                            // New instance of a FramedWriter
                            let mut writer = FramedWriter::new();

                            // Encode a header for the new stream
                            StreamOpenHeader::StardustReliable { channel: VarInt::from_u32(channel.into()) }.encode(&mut scr);
                            writer.queue_raw(Bytes::copy_from_slice(&scr));

                            // Queue all messages in the writer
                            for payload in messages.iter().cloned() {
                                writer.queue_framed(payload);

                                // Try to send in the stream
                                let wresult = writer.write(&mut stream);
                                match (wresult, writer.unsent()) {
                                    // Writer done, move on
                                    (Ok(_), 0) => { continue },

                                    // Writer not done, add to map
                                    (Ok(_), _) => {
                                        framed_writers.insert(stream_id, writer);
                                        continue 'outgoing; // Skip this channel
                                    },

                                    (Err(WriteError::Stopped(_code)), _) => { todo!() },
                                    (Err(WriteError::ClosedStream), _) => { todo!() },

                                    // Doesn't happen, FramedWriter handles it
                                    (Err(WriteError::Blocked), _) => unreachable!(),
                                }
                            }
                        }
                    }
                },
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