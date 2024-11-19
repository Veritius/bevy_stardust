use std::{collections::HashMap, sync::Arc};
use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use tokio::sync::oneshot::error::TryRecvError;

pub struct Endpoint {
    handle: tokio::task::JoinHandle<()>,
    state: tokio::sync::watch::Receiver<EndpointState>,
    close: Option<tokio::sync::oneshot::Sender<()>>,
    wakeup: Arc<tokio::sync::Notify>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn state(&self) -> EndpointState {
        self.state.borrow().clone()
    }

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
    wakeup: Arc<tokio::sync::Notify>,

    connections: HashMap<
        quinn_proto::ConnectionHandle,
        Connection,
    >,

    quinn: quinn_proto::Endpoint,

    quinn_events_rx: tokio::sync::mpsc::UnboundedReceiver<(
        quinn_proto::ConnectionHandle,
        quinn_proto::EndpointEvent,
    )>,
}

struct Connection {
    wakeup: Arc<tokio::sync::Notify>,

    quinn_events_tx: tokio::sync::mpsc::UnboundedSender<
        quinn_proto::ConnectionEvent,
    >,
}

async fn run(
    mut state: State,
) {
    loop {
        tick(&mut state).await;
        state.wakeup.notified().await;
    }
}

async fn tick(
    state: &mut State,
) {
    // Iterate over incoming connection events
    while let Ok((handle, event)) = state.quinn_events_rx.try_recv() {
        if let Some(event) = state.quinn.handle_event(handle, event) {
            let connection = state.connections.get(&handle).unwrap(); // TODO: Handle error
            connection.quinn_events_tx.send(event).unwrap(); // TODO: Handle error
        }
    }

    // See if the closer has been fired
    match state.closer.try_recv() {
        Ok(()) => todo!(),
        Err(TryRecvError::Empty) => { /* Do nothing */ },
        Err(TryRecvError::Closed) => todo!(),
    }
}