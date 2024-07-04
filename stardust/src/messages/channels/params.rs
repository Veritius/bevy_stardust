use std::{any::TypeId, marker::PhantomData, sync::Arc, ops::Deref};
use bevy::ecs::{component::Tick, system::{SystemMeta, SystemParam}, world::unsafe_world_cell::UnsafeWorldCell};
use super::registry::*;
use super::*;

/// Access to registered channels and channel data.
/// 
/// This is only available after [`StardustPlugin`]`::`[`cleanup`] is called.
/// Attempts to access this type before this point will cause a panic.
/// 
/// For asynchronous contexts, [`clone_arc`](Self::clone_arc) can be used
/// to get a reference to the registry that will exist longer than the system.
/// This can be used in the [`ComputeTaskPool`] or [`AsyncComputeTaskPool`].
/// 
/// [`StardustPlugin`]: crate::plugin::StardustPlugin
/// [`cleanup`]: bevy::app::Plugin::cleanup
/// [`ComputeTaskPool`]: bevy::tasks::ComputeTaskPool
/// [`AsyncComputeTaskPool`]: bevy::tasks::AsyncComputeTaskPool
#[derive(SystemParam)]
pub struct Channels<'w> {
    // This hides the ChannelRegistryFinished type so that it
    // cannot be removed from the World, which would be bad
    finished: Res<'w, ChannelRegistryFinished>,
}

impl<'w> Channels<'w> {
    /// Returns an `Arc` to the underlying `ChannelRegistry`.
    /// This allows the registry to be used in asynchronous contexts.
    pub fn clone_arc(&self) -> Arc<ChannelRegistry> {
        self.finished.0.clone()
    }
}

impl<'a> AsRef<ChannelRegistry> for Channels<'a> {
    #[inline]
    fn as_ref(&self) -> &ChannelRegistry {
        &self.finished.0
    }
}

impl<'a> Deref for Channels<'a> {
    type Target = ChannelRegistry;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

/// A `SystemParam` that provides rapid, cached access to data about channel `C`.
///
/// Unlike [`Channels`], `ChannelData` accesses data when the system is run by the scheduler.
/// The data that `Channels` returns is cached, allowing fast repeat access.
/// Using `ChannelData` is more convenient if `C` is known at compile time.
/// 
/// # Panics
/// Panics when used as a [`SystemParam`] if `C` is not registered.
/// 
/// If `C` may not be registered, use [`Channels`] instead.
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

impl<'a, C: Channel> Clone for ChannelData<'a, C> {
    fn clone(&self) -> ChannelData<'a, C> {
        Self {
            registration: self.registration,
            phantom: PhantomData,
        }
    }
}

impl<'a, C: Channel> Copy for ChannelData<'a, C> {}

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
    type Item<'world, 'state> = ChannelData<'world, C>;

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
        let registry = <Res<'world, ChannelRegistryFinished> as SystemParam>::get_param(
            &mut state.res_state,
            system_meta,
            world,
            change_tick
        ).into_inner();

        return ChannelData {
            registration: registry.get_registration(state.channel).unwrap(),
            phantom: PhantomData,
        }
    }
}