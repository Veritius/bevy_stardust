use std::sync::Arc;
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use quinn_proto::{ConnectionEvent, EndpointEvent};
use tokio::sync::{mpsc, Mutex, Notify};

pub struct Connection {
    inner: ConnectionRef,
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(_hooks: &mut ComponentHooks) {}
}

#[derive(Clone)]
pub(crate) struct ConnectionRef(Arc<EndpointInner>);

struct EndpointInner {
    state: Mutex<State>,
    shared: Shared,
}

struct State {
    quinn: quinn_proto::Connection,
}

struct Shared {
    runtime: tokio::runtime::Handle,
    wakeup: Notify,

    endpoint: EndpointHandle,
}

struct EndpointHandle {
    quinn_event_tx: mpsc::UnboundedSender<EndpointEvent>,
    quinn_event_rx: mpsc::UnboundedReceiver<ConnectionEvent>,
}