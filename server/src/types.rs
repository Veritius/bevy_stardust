use bevy_stardust_shared::bevy_ecs::system::Resource;
use bevy_stardust_shared::bevy_ecs;
use bevy_stardust_shared::types::NetworkTypeId;

/// Serverside type storage.
#[derive(Resource)]
pub struct NetworkTypeStorage {
    
}

pub struct NetworkTypeStorageBuilder {
    last_id: u32,
    associations: Vec<(String, NetworkTypeId)>,
}