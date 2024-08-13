use std::{collections::{BTreeMap, VecDeque}, time::Instant};
use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use bevy_stardust::prelude::*;
use bevy_stardust_quic::{RecvStreamId, SendContext, SendStreamId};
use quinn_proto::{ConnectionEvent as QuinnConnectionEvent, ConnectionHandle, Dir, EndpointEvent, SendStream, StreamId as QuinnStreamId, Transmit, WriteError};
use crate::Endpoint;

/// A QUIC connection using `quinn_proto`.
/// 
/// # Safety
/// A [`Connection`] component being removed from the [`World`] it was created in,
/// then being added to a different [`World`], is undefined behavior.
#[derive(Reflect)]
#[reflect(from_reflect=false, Component)]
pub struct Connection {
    #[reflect(ignore)]
    endpoint: Option<Entity>,

    #[reflect(ignore)]
    inner: Box<ConnectionInner>,
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Get the component from the world
            let this = &mut *world.get_mut::<Connection>(entity).unwrap();
            let (handle, endpoint) = (this.inner.handle, this.endpoint);

            // Inform the endpoint of the connection being dropped
            if let Some(endpoint) = endpoint {
                let endpoint = &mut *world.get_mut::<Endpoint>(endpoint).unwrap();
                endpoint.detach(handle);
            }
        });
    }
}

impl Connection {
    pub(crate) fn new(
        handle: ConnectionHandle,
        quinn: quinn_proto::Connection,
    ) -> Self {
        Self {
            endpoint: None,

            inner: ConnectionInner::new(
                handle,
                quinn,
            ),
        }
    }

    pub(crate) fn detach(&mut self) {
        self.endpoint = None;
    }

    pub(crate) fn handle_event(&mut self, event: QuinnConnectionEvent) {
        self.inner.quinn.handle_event(event);
    }

    pub(crate) fn handle_timeout(&mut self) {
        self.inner.quinn.handle_timeout(Instant::now())
    }

    pub(crate) fn poll_endpoint_events(&mut self) -> Option<EndpointEvent> {
        self.inner.quinn.poll_endpoint_events()
    }

    pub(crate) fn poll_transmit(
        &mut self,
        buf: &mut Vec<u8>,
    ) -> Option<Transmit> {
        self.inner.quinn.poll_transmit(
            Instant::now(),
            1,
            buf,
        )
    }
}

struct ConnectionInner {
    handle: ConnectionHandle,

    quinn: quinn_proto::Connection,
    qsm: bevy_stardust_quic::Connection,

    qsids_to_ssids: BTreeMap<QuinnStreamId, SendStreamId>,
    ssids_to_qsids: BTreeMap<SendStreamId, QuinnStreamId>,

    stream_write_queues: BTreeMap<QuinnStreamId, StreamWriteQueue>,
}

impl Drop for ConnectionInner {
    fn drop(&mut self) {
        // Check if the component is drained
        if !self.quinn.is_drained() {
            warn!("Connection was dropped while not fully drained");
        }
    }
}

impl ConnectionInner {
    fn new(
        handle: ConnectionHandle,
        quinn: quinn_proto::Connection,
    ) -> Box<Self> {
        Box::new(Self {
            handle,

            quinn,
            qsm: bevy_stardust_quic::Connection::new(),

            qsids_to_ssids: BTreeMap::new(),
            ssids_to_qsids: BTreeMap::new(),

            stream_write_queues: BTreeMap::new(),
        })
    }

    fn drain_quinn_recv_stream(&mut self, id: QuinnStreamId) {
        match self.quinn.recv_stream(id).read(true) {
            Ok(mut chunks) => {
                loop { match chunks.next(1024) {
                    Ok(Some(chunk)) => {
                        // Forward the received chunk to the stream state machine
                        self.qsm.stream_recv(qsid_to_rsid(id), chunk.bytes);
                    },

                    Ok(None) => break,

                    Err(_) => todo!(),
                } };

                let _ = chunks.finalize();
            },

            Err(_) => todo!(),
        };
    }

    fn stream_queue_chunk(&mut self, qsid: QuinnStreamId, chunk: Bytes) -> Result<(), WriteError> {
        let mut stream = self.quinn.send_stream(qsid);

        // If there's a queue in the map, that means there's previous data that must be sent
        // We just add our chunk to the queue, and then try to write that stream anyway.
        if let Some(queue) = self.stream_write_queues.get_mut(&qsid) {
            queue.push(chunk);

            match queue.write(&mut stream) {
                // The queue is fully drained, remove it
                Ok(true) => {
                    self.discard_stream_write_queue(qsid);
                    return Ok(());
                },

                // The queue is not finished, leave it
                Ok(false) => return Ok(()),

                // The stream returned an error while trying to write to it
                Err(err) => return Err(err),
            }
        } else {
            // In this case, there's no queue in the map.
            // Rather than immediately allocating a queue, we try writing the chunk first,
            // and only add it to the queue if the full chunk isn't written to the stream.
            match stream.write(&chunk) {
                // Do nothing, the full chunk was written
                Ok(l) if l == chunk.len() => return Ok(()),

                Ok(l) => {
                    // Take a slice that excludes the written portion, and add it to the queue
                    let mut queue = StreamWriteQueue::new();
                    queue.push(chunk.slice(l..));
                    self.stream_write_queues.insert(qsid, queue);
                    return Ok(())
                }

                // The stream returned an error while trying to write to it
                Err(err) => return Err(err),
            }
        }
    }

    fn try_drain_stream_queue(&mut self, qsid: QuinnStreamId) -> Result<(), WriteError> {
        let mut stream = self.quinn.send_stream(qsid);
        if let Some(queue) = self.stream_write_queues.get_mut(&qsid) {
            match queue.write(&mut stream) {
                // The queue is fully drained, remove it
                Ok(true) => {
                    self.discard_stream_write_queue(qsid);
                    return Ok(());
                },

                // The queue is not finished, leave it
                Ok(false) => return Ok(()),

                // The stream returned an error while trying to write to it
                Err(err) => return Err(err),
            }
        }

        // No queue
        return Ok(());
    }

    fn discard_stream_write_queue(&mut self, qsid: QuinnStreamId) {
        self.stream_write_queues.remove(&qsid);
    }
}

pub(crate) fn connection_events_system(
    mut connections: Query<&mut Connection>,
) {
    connections.par_iter_mut().for_each(|mut connection| {
        // Timeouts can produce additional events
        connection.handle_timeout();

        // Borrow the inner connection state
        let connection = &mut *connection.inner;

        // Poll until we run out of events
        while let Some(event) = connection.quinn.poll() {
            match event {
                quinn_proto::Event::Stream(event) => match event {
                    quinn_proto::StreamEvent::Opened { dir } => {
                        let id = connection.quinn.streams().accept(dir)
                            .expect("The Opened stream event was raised, but there were no streams to accept");

                        // Inform the state machine that a new stream was opened
                        connection.qsm.stream_opened(qsid_to_rsid(id));

                        // Try to read the stream immediately
                        connection.drain_quinn_recv_stream(id);
                    },

                    quinn_proto::StreamEvent::Readable { id } => {
                        connection.drain_quinn_recv_stream(id);
                    },

                    quinn_proto::StreamEvent::Finished { id } => {
                        connection.qsm.stream_finished(qsid_to_rsid(id));

                        if let Some(ssid) = connection.qsids_to_ssids.remove(&id) {
                            connection.ssids_to_qsids.remove(&ssid);
                        }
                    },

                    quinn_proto::StreamEvent::Stopped { id, error_code: _ } => {
                        connection.qsm.stream_stopped(qsid_to_ssid(id));

                        if let Some(ssid) = connection.qsids_to_ssids.remove(&id) {
                            connection.ssids_to_qsids.remove(&ssid);
                        }
                    },

                    quinn_proto::StreamEvent::Writable { id } => {
                        if let Err(err) = connection.try_drain_stream_queue(id) {
                            todo!()
                        }
                    },

                    quinn_proto::StreamEvent::Available { dir: _ } => {},
                },

                quinn_proto::Event::DatagramReceived => {
                    let payload = connection.quinn.datagrams().recv().unwrap();
                    connection.qsm.recv_dgram(payload);
                },

                quinn_proto::Event::DatagramsUnblocked => todo!(),

                quinn_proto::Event::Connected => todo!(),
                quinn_proto::Event::ConnectionLost { reason } => todo!(),

                // We don't care about this event
                quinn_proto::Event::HandshakeDataReady => {},
            }
        }
    });
}

pub(crate) fn qsm_events_system(
    mut connections: Query<&mut Connection>,
) {
    connections.par_iter_mut().for_each(|mut connection| {
        // Reborrow Connection because borrowck gets angy with Mut<T>
        let connection = &mut *connection.inner;

        // Event polling loop
        connection.qsm.handle_timeout(Instant::now());
        while let Some(event) = connection.qsm.poll() {
            match event {
                bevy_stardust_quic::ConnectionEvent::StreamEvent(event) => match event {
                    bevy_stardust_quic::StreamEvent::Open { id } => {
                        // TODO: Handle the None case
                        let sid = connection.quinn.streams().open(Dir::Uni).unwrap();
                        connection.qsids_to_ssids.insert(sid, id);
                        connection.ssids_to_qsids.insert(id, sid);
                    },

                    bevy_stardust_quic::StreamEvent::Transmit { id, chunk } => {
                        let qsid = *(connection.ssids_to_qsids.get(&id).unwrap());
                        if let Err(err) = connection.stream_queue_chunk(qsid, chunk) {
                            todo!()
                        }
                    },

                    bevy_stardust_quic::StreamEvent::SetPriority { id, priority } => {
                        let sid = *(connection.ssids_to_qsids.get(&id).unwrap());
                        let mut stream = connection.quinn.send_stream(sid);
                        let priority = (priority as i64) << 2i64.pow(32);
                        stream.set_priority(priority.try_into().unwrap()).unwrap();
                    },

                    bevy_stardust_quic::StreamEvent::Reset { id } => {
                        let sid = *(connection.ssids_to_qsids.get(&id).unwrap());
                        let mut stream = connection.quinn.send_stream(sid);
                        stream.reset(todo!()).unwrap();
                    },

                    bevy_stardust_quic::StreamEvent::Finish { id } => {
                        let sid = *(connection.ssids_to_qsids.get(&id).unwrap());
                        let mut stream = connection.quinn.send_stream(sid);
                        stream.finish().unwrap();
                    },

                    bevy_stardust_quic::StreamEvent::Stop { id } => {
                        let mut stream = connection.quinn.recv_stream(rsid_to_qsid(id));
                        stream.stop(todo!()).unwrap();
                    },
                },

                bevy_stardust_quic::ConnectionEvent::TransmitDatagram(data) => {
                    connection.quinn.datagrams().send(data, true).unwrap();
                },

                bevy_stardust_quic::ConnectionEvent::ReceivedMessage(_) => todo!(),

                bevy_stardust_quic::ConnectionEvent::Overheated => todo!(),
            }
        }
    });
}

pub(crate) fn outgoing_messages_system(
    channels: Channels,
    mut connections: Query<(&mut Connection, &PeerMessages<Outgoing>)>,
) {
    connections.par_iter_mut().for_each(|(mut connection, outgoing)| {
        let context = SendContext {
            registry: channels.as_ref(),
            dgram_max_size: connection.inner.quinn.datagrams().max_size().unwrap(),
        };

        connection.inner.qsm.handle_outgoing(context, outgoing);
    });
}

struct StreamWriteQueue {
    queue: VecDeque<Bytes>,
}

impl StreamWriteQueue {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn push(&mut self, chunk: Bytes) {
        self.queue.push_back(chunk);
    }

    fn write(&mut self, stream: &mut SendStream) -> Result<bool, WriteError> {
        // Pop from queue
        if let Some(chunk) = self.queue.pop_front() {
            // Try to write
            match stream.write(&chunk[..]) {
                // Chunk was fully written
                Ok(l) if l == chunk.len() => {
                    // Return whether or not the queue is drained
                    let drained = self.queue.len() == 0;
                    return Ok(drained);
                },

                // Chunk was partially written
                Ok(l) => {
                    // Remove the written portion and put it back
                    let slice = chunk.slice(l..);
                    self.queue.push_front(slice);

                    // Partial writes mean the queue is not drained
                    return Ok(false);
                }

                // No writing was possible due to congestion
                Err(WriteError::Blocked) => {
                    // Put the item back into the queue
                    self.queue.push_front(chunk);
                    return Ok(false);
                }

                // Error while writing
                Err(err) => return Err(err),
            }
        }

        // Queue is drained
        return Ok(true);
    }
}

#[inline]
fn qsid_to_rsid(id: QuinnStreamId) -> RecvStreamId {
    RecvStreamId(id.0)
}

#[inline]
fn qsid_to_ssid(id: QuinnStreamId) -> SendStreamId {
    SendStreamId(id.0)
}

#[inline]
fn rsid_to_qsid(id: RecvStreamId) -> QuinnStreamId {
    QuinnStreamId(id.0)
}

#[inline]
fn ssid_to_qsid(id: SendStreamId) -> QuinnStreamId {
    QuinnStreamId(id.0)
}