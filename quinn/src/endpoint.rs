use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Instant};
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use quinn_proto::{ClientConfig, ConnectError, ConnectionHandle as QuinnHandle, EndpointConfig, EndpointEvent, ServerConfig};
use crate::socket::QuicSocket;

/// A QUIC endpoint.
pub struct Endpoint {
    connections: EndpointConnections,
    inner: Box<EndpointInner>,
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
            connections: EndpointConnections::new(),

            inner: Box::new(EndpointInner::new(
                socket,
                config,
                server,
            )),
        }
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
}

struct EndpointConnections {
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
}