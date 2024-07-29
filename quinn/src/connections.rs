use std::{collections::BTreeMap, time::Instant};
use bevy::{ecs::component::{ComponentHooks, StorageType}, prelude::*};
use bevy_stardust_quic::{RecvStreamId, SendStreamId};
use quinn_proto::{ConnectionEvent as QuinnConnectionEvent, ConnectionHandle, Dir, EndpointEvent, StreamId as QuinnStreamId};
use crate::Endpoint;

/// A QUIC connection using `quinn_proto`.
/// 
/// # Safety
/// A [`Connection`] component being removed from the [`World`] it was created in,
/// then being added to a different [`World`], is undefined behavior.
pub struct Connection {
    inner: Box<ConnectionInner>,
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            // Get the component from the world
            let this = &*world.get::<Connection>(entity).unwrap().inner;

            // Check if the component is drained
            if !this.quinn.is_drained() {
                warn!("Connection {entity} was dropped while not fully drained");
            }

            // Inform the endpoint of the connection being dropped
            let (endpoint, handle) = (this.endpoint, this.handle);
            if let Some(mut endpoint) = world.get_mut::<Endpoint>(endpoint) {
                endpoint.remove_connection(handle);
            }
        });
    }
}

impl Connection {
    pub(crate) fn handle_event(&mut self, event: QuinnConnectionEvent) {
        self.inner.quinn.handle_event(event);
    }

    pub(crate) fn handle_timeout(&mut self) {
        self.inner.quinn.handle_timeout(Instant::now())
    }

    pub(crate) fn poll_endpoint_events(&mut self) -> Option<EndpointEvent> {
        self.inner.quinn.poll_endpoint_events()
    }
}

struct ConnectionInner {
    endpoint: Entity,

    handle: ConnectionHandle,

    quinn: quinn_proto::Connection,
    qsm: bevy_stardust_quic::Connection,

    qsids_to_ssids: BTreeMap<QuinnStreamId, SendStreamId>,
    ssids_to_qsids: BTreeMap<SendStreamId, QuinnStreamId>,

    #[cfg(debug_assertions)]
    world: bevy::ecs::world::WorldId,
}

impl ConnectionInner {
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
                        todo!()
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

        let mut iter = connection.qsm.poll(Instant::now());

        while let Some(event) = iter.next() {
            match event {
                bevy_stardust_quic::ConnectionEvent::StreamEvent(event) => match event {
                    bevy_stardust_quic::StreamEvent::Open { id } => {
                        // TODO: Handle the None case
                        let sid = connection.quinn.streams().open(Dir::Uni).unwrap();
                        connection.qsids_to_ssids.insert(sid, id);
                        connection.ssids_to_qsids.insert(id, sid);
                    },

                    bevy_stardust_quic::StreamEvent::Transmit { id, chunk } => {
                        let sid = *(connection.ssids_to_qsids.get(&id).unwrap());
                        let mut stream = connection.quinn.send_stream(sid);

                        todo!()
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

pub(crate) mod token {
    use super::*;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) struct ConnectionOwnershipToken(Entity);

    impl ConnectionOwnershipToken {
        /// Creates a new [`ConnectionOwnershipToken`] from an [`Entity`] identifier.
        /// 
        /// # SAFETY
        /// There must only be one token for one `id` value in the `World`.
        pub unsafe fn new(id: Entity) -> Self {
            Self(id)
        }

        #[inline]
        pub fn inner(&self) -> Entity {
            self.0
        }
    }

    impl PartialEq<Entity> for ConnectionOwnershipToken {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.0.eq(other)
        }
    }

    impl PartialEq<ConnectionOwnershipToken> for Entity {
        #[inline]
        fn eq(&self, other: &ConnectionOwnershipToken) -> bool {
            self.eq(&other.0)
        }
    }

    impl From<&ConnectionOwnershipToken> for Entity {
        #[inline]
        fn from(value: &ConnectionOwnershipToken) -> Self {
            value.inner()
        }
    }
}

#[cfg(debug_assertions)]
pub(crate) fn safety_check_system(
    world: bevy::ecs::world::WorldId,
    connections: Query<&Connection>,
) {
    for connection in &connections {
        assert_eq!(connection.inner.world, world,
            "A Connection had a world ID different from the one it was created in. This is undefined behavior!");
    }
}