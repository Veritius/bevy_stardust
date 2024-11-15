use bevy_ecs::component::{Component, ComponentHooks, StorageType};

pub struct Endpoint {
    handle: tokio::task::JoinHandle<()>,
    state: tokio::sync::watch::Receiver<EndpointState>,
    close: Option<tokio::sync::oneshot::Sender<()>>,
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

pub enum EndpointState {
    Established,
    Closed,
}

async fn endpoint_task(
    handle: tokio::runtime::Handle,
    quinn: quinn_proto::Endpoint,
    socket: tokio::net::UdpSocket,

    state: tokio::sync::watch::Sender<EndpointState>,
    closer: tokio::sync::oneshot::Receiver<()>,
) {

}