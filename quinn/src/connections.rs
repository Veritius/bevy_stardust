use std::time::Instant;
use bevy::prelude::*;
use bevy_stardust_quic::{RecvStreamId, SendStreamId};
use quinn_proto::{ConnectionEvent as QuinnConnectionEvent, ConnectionHandle, EndpointEvent, StreamId as QuinnStreamId};

/// A QUIC connection using `quinn_proto`.
/// 
/// # Safety
/// This component must not be moved from the [`World`] it was originally added to.
#[derive(Component)]
pub struct Connection {
    endpoint: Entity,

    handle: ConnectionHandle,

    quinn: quinn_proto::Connection,
    qsm: bevy_stardust_quic::Connection,

    #[cfg(debug_assertions)]
    world: bevy::ecs::world::WorldId,
}

impl Connection {
    pub(crate) fn handle_event(&mut self, event: QuinnConnectionEvent) {
        self.quinn.handle_event(event);
    }

    pub(crate) fn handle_timeout(&mut self) {
        self.quinn.handle_timeout(Instant::now())
    }

    pub(crate) fn poll_endpoint_events(&mut self) -> Option<EndpointEvent> {
        self.quinn.poll_endpoint_events()
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
}

pub(crate) fn connection_events_system(
    mut connections: Query<&mut Connection>,
) {
    connections.par_iter_mut().for_each(|mut connection| {
        // Timeouts can produce additional events
        connection.handle_timeout();

        // Poll until we run out of events
        while let Some(event) = connection.quinn.poll() {
            match event {
                quinn_proto::Event::Stream(event) => match event {
                    quinn_proto::StreamEvent::Opened { dir } => {
                        let id = connection.quinn.streams().accept(dir)
                            .expect("The Opened stream event was raised, but there were no streams to accept");

                        todo!()
                    },

                    quinn_proto::StreamEvent::Readable { id } => {
                        connection.drain_quinn_recv_stream(id);
                    },

                    quinn_proto::StreamEvent::Writable { id } => todo!(),
                    quinn_proto::StreamEvent::Finished { id } => todo!(),
                    quinn_proto::StreamEvent::Stopped { id, error_code } => todo!(),
                    quinn_proto::StreamEvent::Available { dir } => todo!(),
                },

                quinn_proto::Event::Connected => todo!(),
                quinn_proto::Event::ConnectionLost { reason } => todo!(),

                quinn_proto::Event::DatagramReceived => todo!(),
                quinn_proto::Event::DatagramsUnblocked => todo!(),

                // We don't care about this event
                quinn_proto::Event::HandshakeDataReady => {},
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
    #[repr(transparent)]
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
        assert_eq!(connection.world, world,
            "A Connection had a world ID different from the one it was created in. This is undefined behavior!");
    }
}