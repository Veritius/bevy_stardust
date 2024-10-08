use bevy_ecs::{component::{ComponentHooks, StorageType}, prelude::*};
use quinn_proto::ConnectionHandle as QuinnHandle;

/// A QUIC connection.
pub struct Connection(pub(crate) Box<ConnectionInner>);

impl Component for Connection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_remove(|mut world, entity, _| {
            todo!()
        });
    }
}

pub(crate) struct ConnectionInner {
    handle: QuinnHandle,
}