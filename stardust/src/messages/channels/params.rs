use std::{any::TypeId, marker::PhantomData};
use bevy::ecs::{component::Tick, system::{SystemMeta, SystemParam}, world::unsafe_world_cell::UnsafeWorldCell};
use super::registry::*;
use super::*;

/// A `SystemParam` that gives shorthand access to data about the channel `C`.
pub struct ChannelData<'a, C: Channel> {
    registration: &'a Registration,
    phantom: PhantomData<C>,
}

impl<C: Channel> ChannelData<'_, C> {
    /// Returns the [`ChannelId`] assigned to `C`.
    #[inline]
    pub fn id(&self) -> ChannelId {
        self.metadata().channel_id
    }

    /// Returns the [`ChannelMetadata`] of channel `C`.
    #[inline]
    pub fn metadata(&self) -> &ChannelMetadata {
        &self.registration.metadata
    }

    /// Returns the [`ChannelConfiguration`] of channel `C`.
    #[inline]
    pub fn config(&self) -> &ChannelConfiguration {
        &self.registration.config
    }
}

pub struct ChannelDataState {
    // Directly use the State type from the SystemParam implementation
    // This avoids type errors if it's changed in future. It shouldn't, but eh.
    // The lifetime should be irrelevant here. If it isn't, a type error is thrown.
    res_state: <Res<'static, ChannelRegistryFinished> as SystemParam>::State,
    channel: ChannelId,
}

unsafe impl<C> SystemParam for ChannelData<'_, C>
where
    C: Channel,
{
    type State = ChannelDataState;
    type Item<'world, 'state> = Self;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        let res_state = <Res<ChannelRegistryFinished> as SystemParam>::init_state(world, system_meta);
        let registry = world.resource::<ChannelRegistryFinished>();
        let channel = registry.id(TypeId::of::<C>()).unwrap();
        return ChannelDataState { res_state, channel };
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        change_tick: Tick,
    ) -> Self::Item<'world, 'state> {
        let registry = <Res<ChannelRegistryFinished> as SystemParam>::get_param(
            &mut state.res_state,
            system_meta,
            world,
            change_tick
        );

        todo!()

        // return ChannelData::<'state> {
        //     registration: registry.get_registration(state.channel).unwrap(),
        //     phantom: PhantomData,
        // }
    }
}