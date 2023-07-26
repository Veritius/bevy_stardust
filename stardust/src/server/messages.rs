use std::marker::PhantomData;
use bevy::{prelude::{Resource, Res}, ecs::system::SystemParam};
use crate::shared::{channel::Channel, protocol::Protocol};

#[derive(Resource)]
pub(super) struct UntypedChannelStore {

}

/// Used to read payload information for a channel.
#[derive(SystemParam)]
pub struct ChannelReader<'w, T: Channel> {
    store: Res<'w, UntypedChannelStore>,
    protocol: Res<'w, Protocol>,
    phantom: PhantomData<T>,
}