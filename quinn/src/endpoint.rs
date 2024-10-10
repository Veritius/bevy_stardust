use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Instant};
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use quinn_proto::{ClientConfig, ConnectError, ConnectionEvent, ConnectionHandle as QuinnHandle, EndpointConfig, EndpointEvent, ServerConfig};
use crate::{access::ParEndpoints, socket::{BoundUdpSocket, QuicSocket, Transmit}};

/// A QUIC endpoint.
pub struct Endpoint {
    inner: Box<EndpointInner>,

    connections: EndpointConnections,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            todo!()
        });
    }
}

impl Endpoint {
    pub(crate) fn new(
        socket: QuicSocket,
        config: Arc<EndpointConfig>,
        server: Option<Arc<ServerConfig>>,
    ) -> Self {
        Self {
            inner: Box::new(EndpointInner::new(
                socket,
                config,
                server,
            )),

            connections: EndpointConnections::new(),
        }
    }

    pub(crate) fn split_access(&mut self) -> (
        &mut EndpointInner,
        &EndpointConnections,
    ) {
        (
            &mut *self.inner,
            &self.connections,
        )
    }

    pub(crate) unsafe fn inform_connection_close(
        &mut self,
        entity: Entity,
    ) {
        // Remove the handle from the connection map.
        let handle = match self.connections.remove_by_entity(entity) {
            Some(handle) => handle,
            None => return,
        };

        // Inform the Quinn state machine that the endpoint has been removed.
        self.inner.endpoint.handle_event(handle, EndpointEvent::drained());
    }

    pub(crate) unsafe fn init_remote_connection(
        &mut self,
        entity: Entity,
        config: ClientConfig,
        address: SocketAddr,
        server_name: &str,
    ) -> Result<(QuinnHandle, quinn_proto::Connection), ConnectError> {
        let (handle, connection) = self.inner.endpoint.connect(
            Instant::now(),
            config,
            address,
            server_name,
        )?;

        // Add to connection map
        self.connections.insert(entity, handle);

        return Ok((handle, connection));
    }
}

pub(crate) struct EndpointInner {
    socket: QuicSocket,

    endpoint: quinn_proto::Endpoint,
}

impl EndpointInner {
    pub fn new(
        socket: QuicSocket,
        config: Arc<EndpointConfig>,
        server: Option<Arc<ServerConfig>>,
    ) -> Self {
        Self {
            socket,

            endpoint: quinn_proto::Endpoint::new(
                config,
                server,
                true,
                None,
            ),
        }
    }

    #[inline]
    pub fn handle_event(
        &mut self,
        handle: QuinnHandle,
        event: EndpointEvent
    ) -> Option<ConnectionEvent> {
        self.endpoint.handle_event(
            handle,
            event
        )
    }
}

pub(crate) struct EndpointConnections {
    e2h: BTreeMap<Entity, QuinnHandle>,
    h2e: BTreeMap<QuinnHandle, Entity>,
}

impl EndpointConnections {
    fn new() -> Self {
        Self {
            e2h: BTreeMap::new(),
            h2e: BTreeMap::new(),
        }
    }

    unsafe fn insert(&mut self, entity: Entity, handle: QuinnHandle) {
        self.e2h.insert(entity, handle);
        self.h2e.insert(handle, entity);
    }

    unsafe fn remove_by_entity(&mut self, entity: Entity) -> Option<QuinnHandle> {
        let handle = self.e2h.remove(&entity)?;
        self.h2e.remove(&handle);
        return Some(handle);
    }

    unsafe fn remove_by_handle(&mut self, handle: QuinnHandle) -> Option<Entity> {
        let entity = self.h2e.remove(&handle)?;
        self.e2h.remove(&entity);
        return Some(entity);
    }

    pub fn get_entity(&self, handle: QuinnHandle) -> Option<Entity> {
        self.h2e.get(&handle).cloned()
    }

    pub fn get_handle(&self, entity: Entity) -> Option<QuinnHandle> {
        self.e2h.get(&entity).cloned()
    }
}

impl<'a> IntoIterator for &'a EndpointConnections {
    type Item = (Entity, QuinnHandle);
    type IntoIter = EndpointConnectionsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

pub(crate) struct EndpointConnectionsIter<'a> {
    iter: std::collections::btree_map::Iter<'a, Entity, QuinnHandle>,
}

impl<'a> Iterator for EndpointConnectionsIter<'a> {
    type Item = (Entity, QuinnHandle);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(a,b)| (*a, *b))
    }
}

pub(crate) fn io_udp_recv_system(
    mut parallel_iterator: ParEndpoints,
) {
    parallel_iterator.iter(|endpoint, connections| {
        let mut scratch = Vec::with_capacity(1472); // TODO make configurable

        'outer: loop {
            match endpoint.endpoint.socket.recv(&mut scratch[..]) {
                Ok(Some(recv)) => {
                    todo!()
                },
    
                Ok(None) => {
                    break 'outer; // no more to receive
                },
    
                Err(_) => todo!(),
            }
        }
    });
}

pub(crate) fn io_udp_send_system(
    mut parallel_iterator: ParEndpoints,
) {
    parallel_iterator.iter(|endpoint, mut connections| {
        let mut scratch = Vec::with_capacity(1472); // TODO make configurable

        for connection_access in connections.iter() {
            while let Some(transmit) = connection_access.connection.poll_transmit(&mut scratch) {
                match endpoint.endpoint.socket.send(Transmit {
                    payload: &scratch[..transmit.size],
                    address: transmit.destination,
                }) {
                    Ok(_) => {}, // TODO

                    Err(_) => todo!(),
                }
            }
        }
    });
}