use bevy_stardust_shared::types::NetworkTypeId;
use bevy::prelude::*;

/// Serverside type storage.
#[derive(Resource)]
pub struct NetworkTypeStorage {
    
}

pub struct NetworkTypeStorageBuilder {
    last_id: u32,
    associations: Vec<(String, NetworkTypeId)>,
}