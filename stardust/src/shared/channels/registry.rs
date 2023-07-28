use bevy::prelude::Resource;
use super::id::{Channel, ChannelId};

#[derive(Resource)]
pub struct ChannelRegistry {

}

impl ChannelRegistry {
    fn add_channel<T: Channel>() -> ChannelId {
        todo!()
    }
}

pub trait ChannelRegistryAppExts {

}