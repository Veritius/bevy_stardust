use bevy::{prelude::*, utils::HashMap};

#[derive(Resource)]
pub(crate) struct NetworkEntityMap {
    map_out: HashMap<Entity, NetworkEntityId>,
    map_in: HashMap<NetworkEntityId, Entity>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NetworkEntityId(u32);