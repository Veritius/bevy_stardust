use std::marker::PhantomData;
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use crate::shared::{channel::Channel, protocol::Protocol};
use super::clients::Client;

/// Sends messages over a network channel to clients.
#[derive(SystemParam)]
pub struct ChannelWriter<'w, 's, T: Channel> {
    clients: Query<'w, 's, &'static Client>,
    protocol: Res<'w, Protocol>,
    phantom: PhantomData<T>,
}