use std::marker::PhantomData;
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::shared::channels::id::Channel;
use crate::shared::channels::registry::ChannelRegistry;
use super::clients::Client;

/// Sends messages over a network channel to clients.
#[derive(SystemParam)]
pub struct ChannelWriter<'w, 's, T: Channel> {
    clients: Query<'w, 's, &'static Client>,
    channel_registry: Res<'w, ChannelRegistry>,
    phantom: PhantomData<T>,
}