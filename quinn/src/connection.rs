use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};

/// A QUIC connection.
pub struct Connection {

}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            todo!()
        });
    }
}