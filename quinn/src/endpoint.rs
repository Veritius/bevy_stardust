use std::collections::BTreeMap;
use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use quinn_proto::ConnectionHandle as QuinnHandle;
use crate::socket::QuicSocket;

/// A QUIC endpoint.
pub struct Endpoint(pub(crate) Box<EndpointInner>);

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            todo!()
        });
    }
}

pub(crate) struct EndpointInner {
    socket: QuicSocket,

    endpoint: quinn_proto::Endpoint,

    connections: EndpointConnections,
}

struct EndpointConnections {
    e2h: BTreeMap<Entity, QuinnHandle>,
    h2e: BTreeMap<QuinnHandle, Entity>,
}