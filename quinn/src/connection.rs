use std::{collections::{BTreeMap, VecDeque}, time::Instant};
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use bevy_stardust::prelude::*;
use bevy_stardust_quic::{RecvStreamId, SendContext, SendStreamId};
use bytes::Bytes;
use quinn_proto::{ConnectionError, ConnectionEvent as QuinnEvent, ConnectionHandle as QuinnHandle, Dir, EndpointEvent, Event as ApplicationEvent, ReadError, StreamEvent as QuinnStreamEvent, StreamId as QuinnStreamId, Transmit, VarInt as QuinnVarInt, WriteError};
use crate::{write_queue::StreamWriteQueue, Endpoint};

/// A QUIC connection.
pub struct Connection(pub(crate) Box<ConnectionInner>);

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Get this entity from the world.
            let this = match world.get_entity(entity) {
                Some(endpoint) => endpoint,
                None => return,
            };

            // Try to get the endpoint entity.
            let endpoint = this.get::<Connection>().unwrap().0.endpoint;
            let mut endpoint = match world.get_entity_mut(endpoint) {
                Some(endpoint) => endpoint,
                None => return,
            };

            // Try to access the endpoint component.
            if let Some(mut endpoint) = endpoint.get_mut::<Endpoint>() {
                // Inform the endpoint of the connection being closed
                unsafe { endpoint.inform_connection_close(entity) };
            }
        });
    }
}

pub(crate) struct ConnectionInner {
    handle: QuinnHandle,
    endpoint: Entity,

    quinn: quinn_proto::Connection,
    write_queues: BTreeMap<QuinnStreamId, StreamWriteQueue>,

    statemachine: bevy_stardust_quic::Connection,
    map_qsid_ssid: BTreeMap<QuinnStreamId, SendStreamId>,
    map_ssid_qsid: BTreeMap<SendStreamId, QuinnStreamId>,

    events: VecDeque<ConnectionEvent>,
    messages: VecDeque<ChannelMessage>,
}

impl ConnectionInner {
    pub unsafe fn new(
        handle: QuinnHandle,
        endpoint: Entity,
        connection: quinn_proto::Connection,
        statemachine: bevy_stardust_quic::Connection,
    ) -> Self {
        Self {
            handle,
            endpoint,

            quinn: connection,
            write_queues: BTreeMap::new(),

            statemachine,
            map_qsid_ssid: BTreeMap::new(),
            map_ssid_qsid: BTreeMap::new(),

            events: VecDeque::new(),
            messages: VecDeque::new(),
        }
    }

    pub fn close(
        &mut self
    ) {
        self.quinn.close(
            Instant::now(),
            QuinnVarInt::from_u32(0),
            Bytes::new(),
        );
    }

    #[inline]
    pub fn handle(&self) -> QuinnHandle {
        self.handle
    }

    pub fn handle_timeouts(
        &mut self,
        now: Instant,
    ) {
        self.quinn.handle_timeout(now);
        self.statemachine.handle_timeout(now);
    }

    pub fn handle_connection_event(
        &mut self,
        event: QuinnEvent,
    ) {
        self.quinn.handle_event(event);

        while let Some(event) = self.quinn.poll() {
            match event {
                ApplicationEvent::Stream(stream_event) => match stream_event {
                    QuinnStreamEvent::Opened { dir } => {
                        let id = self.quinn.streams().accept(dir)
                            .expect("A stream was reported to be open in an event, but accepting it failed");

                        self.try_recv_from_stream(id);
                    },

                    QuinnStreamEvent::Readable { id } => {
                        self.try_recv_from_stream(id);
                    },

                    QuinnStreamEvent::Writable { id } => {
                        self.try_drain_write_queue(id)
                            .expect("Tried to write but an error was encountered");
                    },

                    QuinnStreamEvent::Finished { id } => {
                        self.statemachine.stream_finished(qsid_to_rsid(id));

                        if let Some(ssid) = self.map_qsid_ssid.remove(&id) {
                            self.map_ssid_qsid.remove(&ssid);
                        }
                    },

                    QuinnStreamEvent::Stopped { id, error_code: _ } => {
                        if let Some(ssid) = self.map_qsid_ssid.remove(&id) {
                            self.map_ssid_qsid.remove(&ssid);
                            self.statemachine.stream_stopped(ssid);
                        }
                    },

                    QuinnStreamEvent::Available { dir } => {},
                },

                ApplicationEvent::DatagramReceived => {
                    let payload = self.quinn.datagrams().recv().unwrap();
                    self.statemachine.recv_dgram(payload);
                },

                ApplicationEvent::DatagramsUnblocked => {},

                ApplicationEvent::Connected => {
                    self.events.push_back(ConnectionEvent::Connected);
                },

                ApplicationEvent::ConnectionLost { reason } => {
                    self.events.push_back(ConnectionEvent::Disconnected {
                        reason,
                    });
                },

                // Do nothing here
                ApplicationEvent::HandshakeDataReady => {},
            }
        }

        if self.quinn.is_closed() { return }
        self.handle_interstage_events();
    }

    fn handle_interstage_events(&mut self) {
        while let Some(event) = self.statemachine.poll() {
            match event {
                bevy_stardust_quic::ConnectionEvent::StreamEvent(stream_event) => match stream_event {
                    bevy_stardust_quic::StreamEvent::Open { id } => {
                        let sid = self.quinn.streams().open(Dir::Uni).unwrap();
                        self.map_qsid_ssid.insert(sid, id);
                        self.map_ssid_qsid.insert(id, sid);
                    },

                    bevy_stardust_quic::StreamEvent::Transmit { id, chunk } => {
                        let qsid = *(self.map_ssid_qsid.get(&id).unwrap());

                        if let Err(err) = self.try_write_to_stream(qsid, chunk) {
                            #[cfg(feature="log")]
                            bevy_log::error!("Failed to write to stream {qsid}: {err}");
                        }
                    },

                    bevy_stardust_quic::StreamEvent::SetPriority { id, priority } => {
                        let sid = *(self.map_ssid_qsid.get(&id).unwrap());
                        let mut stream = self.quinn.send_stream(sid);
                        let priority = (priority as i64) << 2i64.pow(32);
                        stream.set_priority(priority.try_into().unwrap()).unwrap();
                    },

                    bevy_stardust_quic::StreamEvent::Reset { id, code } => {
                        let sid = *(self.map_ssid_qsid.get(&id).unwrap());
                        let mut stream = self.quinn.send_stream(sid);
                        stream.reset(QuinnVarInt::from(Into::<u32>::into(code))).unwrap();
                    },

                    bevy_stardust_quic::StreamEvent::Finish { id } => {
                        let sid = *(self.map_ssid_qsid.get(&id).unwrap());
                        let mut stream = self.quinn.send_stream(sid);
                        stream.finish().unwrap();
                    },

                    bevy_stardust_quic::StreamEvent::Stop { id, code } => {
                        let mut stream = self.quinn.recv_stream(rsid_to_qsid(id));
                        stream.stop(QuinnVarInt::from(Into::<u32>::into(code))).unwrap();
                    },
                },

                bevy_stardust_quic::ConnectionEvent::TransmitDatagram(bytes) => {
                    self.quinn.datagrams().send(bytes, true).unwrap();
                },

                bevy_stardust_quic::ConnectionEvent::ReceivedMessage(channel_message) => {
                    self.messages.push_back(channel_message);
                },

                bevy_stardust_quic::ConnectionEvent::Disconnect(code) => {
                    self.quinn.close(
                        Instant::now(),
                        QuinnVarInt::from(Into::<u32>::into(code)),
                        Bytes::new(),
                    );
                },
            }
        }
    }

    #[inline]
    pub fn poll_endpoint_events(
        &mut self,
    ) -> Option<EndpointEvent> {
        self.quinn.poll_endpoint_events()
    }

    #[inline]
    pub fn poll_connection_events(
        &mut self,
    ) -> Option<ConnectionEvent> {
        self.events.pop_front()
    }

    #[inline]
    pub fn poll_messages(
        &mut self,
    ) -> Option<ChannelMessage> {
        self.messages.pop_front()
    }

    pub fn poll_transmit(
        &mut self,
        buf: &mut Vec<u8>,
    ) -> Option<Transmit> {
        self.quinn.poll_transmit(
            Instant::now(),
            1,
            buf,
        )
    }

    pub fn handle_outgoing(
        &mut self,
        context: SendContext,
        outgoing: &PeerMessages<Outgoing>,
    ) {
        self.statemachine.handle_outgoing(
            context,
            &outgoing,
        );
    }

    fn try_recv_from_stream(&mut self, qsid: QuinnStreamId) {
        match self.quinn.recv_stream(qsid).read(true) {
            Ok(mut chunks) => {
                loop { match chunks.next(1024) {
                    Ok(Some(chunk)) => {
                        // Forward the received chunk to the stream state machine
                        self.statemachine.stream_recv(qsid_to_rsid(qsid), chunk.bytes);
                    },

                    // No more to read
                    Ok(None) | Err(ReadError::Blocked) => break,

                    Err(_) => todo!(),
                } };

                let _ = chunks.finalize();
            },

            Err(_) => todo!(),
        };
    }

    fn try_write_to_stream(&mut self, qsid: QuinnStreamId, chunk: Bytes) -> Result<(), WriteError> {
        let mut stream = self.quinn.send_stream(qsid);
        // If there's a queue in the map, that means there's previous data that must be sent
        // We just add our chunk to the queue, and then try to write that stream anyway.
        if let Some(queue) = self.write_queues.get_mut(&qsid) {
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
                    self.write_queues.insert(qsid, queue);
                    return Ok(())
                }

                // The stream returned an error while trying to write to it
                Err(err) => return Err(err),
            }
        }
    }

    fn try_drain_write_queue(&mut self, qsid: QuinnStreamId) -> Result<(), WriteError> {
        let mut stream = self.quinn.send_stream(qsid);
        if let Some(queue) = self.write_queues.get_mut(&qsid) {
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
        self.write_queues.remove(&qsid);
    }
}

pub(crate) enum ConnectionEvent {
    Connected,

    Disconnected {
        reason: ConnectionError,
    }
}

#[inline]
pub(crate) fn qsid_to_rsid(id: QuinnStreamId) -> RecvStreamId {
    RecvStreamId(id.0)
}

#[inline]
pub(crate) fn rsid_to_qsid(id: RecvStreamId) -> QuinnStreamId {
    QuinnStreamId(id.0)
}

#[derive(Bundle)]
pub(crate) struct ConnectionBundle {
    pub connection: Connection,
    pub incoming: PeerMessages<Incoming>,
    pub outgoing: PeerMessages<Outgoing>,
}

impl ConnectionBundle {
    pub fn new(connection: Connection) -> Self {
        Self {
            connection,

            incoming: PeerMessages::new(),
            outgoing: PeerMessages::new(),
        }
    }
}