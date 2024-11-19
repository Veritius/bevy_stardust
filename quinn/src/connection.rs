use std::sync::Arc;

use bevy_ecs::component::{Component, ComponentHooks, StorageType};
use bevy_stardust::prelude::ChannelMessage;
use tokio::sync::oneshot::error::TryRecvError;

pub struct Connection {
    handle: tokio::task::JoinHandle<()>,
    state: tokio::sync::watch::Receiver<ConnectionState>,
    close: Option<tokio::sync::oneshot::Sender<()>>,
    wakeup: Arc<tokio::sync::Notify>,

    messages_rx: crossbeam_channel::Receiver<ChannelMessage>,
    messages_tx: tokio::sync::mpsc::UnboundedSender<ChannelMessage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConnectionState {
    Connecting,
    Established,
    Closed,
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            let mut entity = world.entity_mut(entity);
            let mut component = entity.get_mut::<Connection>().unwrap();
            component.close();
        });
    }
}

impl Connection {
    pub fn state(&self) -> ConnectionState {
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
    state: tokio::sync::watch::Sender<ConnectionState>,
    wakeup: Arc<tokio::sync::Notify>,

    quinn: quinn_proto::Connection,

    quinn_events_rx: tokio::sync::mpsc::UnboundedReceiver<
        quinn_proto::ConnectionEvent,
    >,

    quinn_events_tx: tokio::sync::mpsc::UnboundedSender<
        quinn_proto::EndpointEvent,
    >,

    messages_tx: crossbeam_channel::Sender<ChannelMessage>,
    messages_rx: tokio::sync::mpsc::UnboundedSender<ChannelMessage>,
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
    // See if the closer has been fired
    match state.closer.try_recv() {
        Ok(()) => todo!(),
        Err(TryRecvError::Empty) => { /* Do nothing */ },
        Err(TryRecvError::Closed) => todo!(),
    }

    // Handle incoming connection events
    while let Ok(event) = state.quinn_events_rx.try_recv() {
        state.quinn.handle_event(event);
    }

    // Poll for the next timeout TODO
    // let timeout = state.quinn.poll_timeout();

    // Poll for endpoint events
    while let Some(event) = state.quinn.poll_endpoint_events() {
        state.quinn_events_tx.send(event).unwrap(); // TODO: Handle error
    }

    // Poll for application facing events
    while let Some(event) = state.quinn.poll() {

    }
}