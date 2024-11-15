use bevy_ecs::component::{Component, ComponentHooks, StorageType};

pub struct Endpoint {
    _p: (),
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

    }
}