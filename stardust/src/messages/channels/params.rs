use std::{any::TypeId, marker::PhantomData};
use bevy::ecs::system::SystemParam;
use super::*;

/// A `SystemParam` that gives shorthand access to data about the channel `C`.
#[derive(SystemParam)]
pub struct ChannelData<'w, 's, C: Channel> {
    cache: Local<'s, Option<ChannelId>>,
    channels: Channels<'w>,
    phantom: PhantomData<C>,
}

impl<C: Channel> ChannelData<'_, '_, C> {
    /// Returns the [`ChannelId`] assigned to `C`.
    pub fn id(&mut self) -> ChannelId {
        if let Some(cached) = *self.cache {
            return cached;
        }

        let id = self.channels.id(TypeId::of::<C>()).unwrap();
        *self.cache = Some(id);
        return id;
    }

    /// Returns the [`ChannelConfiguration`] of channel `C`.
    pub fn config(&mut self) -> &ChannelConfiguration {
        let id = self.id();
        self.channels.config(id).unwrap()
    }
}