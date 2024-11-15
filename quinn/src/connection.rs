use bevy_ecs::component::{Component, ComponentHooks, StorageType};

pub struct Connection {
    _p: (),
}

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        // hooks.on_remove(|mut world, entity, _| {
        //     let mut component = world.entity_mut(entity).get_mut::<Connection>().unwrap();
        // });
    }
}