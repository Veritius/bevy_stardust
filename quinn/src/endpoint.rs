use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use crate::socket::QuicSocket;

/// A QUIC endpoint.
pub struct Endpoint {
    socket: QuicSocket,
}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            todo!()
        });
    }
}