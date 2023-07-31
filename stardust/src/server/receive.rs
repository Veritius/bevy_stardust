use std::marker::PhantomData;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::shared::{channels::{id::Channel, registry::ChannelRegistry}, messages::receive::IncomingNetworkMessages};
use super::clients::Client;

/// Added to a Bevy system to read network messages over channel `T`.
#[derive(SystemParam)]
pub struct ChannelReader<'w, 's, T: Channel> {
    channel_registry: Res<'w, ChannelRegistry>,
    clients: Query<'w, 's, (&'static Client, &'static IncomingNetworkMessages)>,
    phantom: PhantomData<T>,
}