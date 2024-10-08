use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};

/// A QUIC endpoint.
pub struct Endpoint {

}

impl Component for Endpoint {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            todo!()
        });
    }
}