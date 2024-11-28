use std::{collections::HashMap, sync::Arc};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use quinn_proto::{ConnectionEvent, ConnectionHandle as QuinnConnectionId, EndpointEvent};
use tokio::sync::{mpsc, Mutex, Notify};
use crate::connection::ConnectionRef;

pub struct Endpoint {
    inner: EndpointRef,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

#[derive(Clone)]
pub(crate) struct EndpointRef(Arc<EndpointInner>);

struct EndpointInner {
    state: Mutex<State>,
    shared: Shared,
}

struct State {
    quinn: quinn_proto::Endpoint,

    connections: HashMap<QuinnConnectionId, ConnectionHandle>,
}

struct Shared {
    runtime: tokio::runtime::Handle,
    wakeup: Notify,
}

struct ConnectionHandle {
    inner_ref: ConnectionRef,

    event_tx: mpsc::UnboundedSender<ConnectionEvent>,
    event_rx: mpsc::UnboundedReceiver<EndpointEvent>,
}