use std::collections::HashMap;
use bevy_ecs::component::{Component, ComponentHooks, StorageType};

pub struct Endpoint {
    handle: tokio::task::JoinHandle<()>,
    state: tokio::sync::watch::Receiver<EndpointState>,
    close: Option<tokio::sync::oneshot::Sender<()>>,
}

pub enum EndpointState {
    Established,
    Closed,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            let mut entity = world.entity_mut(entity);
            let mut component = entity.get_mut::<Endpoint>().unwrap();
            component.close();
        });
    }
}

impl Endpoint {
    pub fn close(
        &mut self,
    ) {
        // If the event is run already, don't bother
        if self.close.is_none() { return }

        // Send the closer one-shot event
        let mut closer = None;
        std::mem::swap(&mut closer, &mut self.close);
        let closer = closer.unwrap();
        let _ = closer.send(());
    }
}

struct State {
    runtime: tokio::runtime::Handle,
    closer: tokio::sync::oneshot::Receiver<()>,
    state: tokio::sync::watch::Sender<EndpointState>,

    quinn: quinn_proto::Endpoint,

    quinn_events_rx: tokio::sync::mpsc::UnboundedReceiver<(
        quinn_proto::ConnectionHandle,
        quinn_proto::EndpointEvent,
    )>,

    quinn_events_tx: HashMap<
        quinn_proto::ConnectionHandle,
        tokio::sync::mpsc::UnboundedSender<
            quinn_proto::ConnectionEvent,
        >,
    >,
}