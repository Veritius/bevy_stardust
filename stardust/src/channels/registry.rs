//! The channel registry.

use std::{any::TypeId, collections::BTreeMap, ops::{Deref, DerefMut}, sync::Arc};
use bevy_ecs::{component::ComponentId, prelude::*, system::SystemParam};
use crate::prelude::ChannelConfiguration;
use super::{id::{Channel, ChannelId}, ToChannelId};

/// Mutable access to the channel registry, only available during app setup.
#[derive(Resource)]
pub(crate) struct SetupChannelRegistry(pub(crate) Box<ChannelRegistryInner>);

impl Deref for SetupChannelRegistry {
    type Target = ChannelRegistryInner;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SetupChannelRegistry {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Immutable access to the channel registry, only available after app setup.
/// 
/// In almost all cases, you should just use the [`ChannelRegistry`] systemparam.
/// However, this type can be cloned and will point to the same inner value.
/// This makes it useful for asynchronous programming, like in futures.
#[derive(Resource, Clone)]
pub struct FinishedChannelRegistry(pub(crate) Arc<ChannelRegistryInner>);

impl Deref for FinishedChannelRegistry {
    type Target = ChannelRegistryInner;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Access to the configuration of registered channels, at any point.
/// 
/// If you're writing async code, you might want to look at [`FinishedChannelRegistry`].
pub struct ChannelRegistry<'a>(&'a ChannelRegistryInner);

unsafe impl<'a> SystemParam for ChannelRegistry<'a> {
    type State = (ComponentId, ComponentId);
    type Item<'w, 's> = ChannelRegistry<'w>;

    fn init_state(world: &mut World, system_meta: &mut bevy_ecs::system::SystemMeta) -> Self::State {
        // SAFETY: Since we can't register accesses, we do it through Res<T> which can
        (
            <Res<FinishedChannelRegistry> as SystemParam>::init_state(world, system_meta),
            <Res<SetupChannelRegistry> as SystemParam>::init_state(world, system_meta),
        )
    }

    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        _system_meta: &bevy_ecs::system::SystemMeta,
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
        _change_tick: bevy_ecs::component::Tick,
    ) -> Self::Item<'w, 's> {
        if let Some(ptr) = world.get_resource_by_id(state.0) {
            return ChannelRegistry(ptr.deref::<FinishedChannelRegistry>().0.as_ref());
        }

        if let Some(ptr) = world.get_resource_by_id(state.1) {
            return ChannelRegistry(ptr.deref::<SetupChannelRegistry>().0.as_ref());
        }

        panic!("Neither SetupChannelRegistry or FinishedChannelRegistry were present when attempting to create ChannelRegistry")
    }
}

impl ChannelRegistry<'_> {
    /// Gets the id fom the `ToChannelId` implementation.
    #[inline]
    pub fn channel_id(&self, from: impl ToChannelId) -> Option<ChannelId> {
        self.0.channel_id(from)
    }

    /// Gets the channel configuration for `id`.
    #[inline]
    pub fn channel_config(&self, id: impl ToChannelId) -> Option<&ChannelData> {
        self.0.channel_config(id)
    }
}

impl Deref for ChannelRegistry<'_> {
    type Target = ChannelRegistryInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Stores channel configuration data. Accessible through the [`ChannelRegistry`] system parameter.
pub struct ChannelRegistryInner {
    pub(super) channel_type_ids: BTreeMap<TypeId, ChannelId>,
    pub(super) channel_data: Vec<ChannelData>,
}

impl ChannelRegistryInner {
    pub(in crate) fn new() -> Self {
        Self {
            channel_type_ids: BTreeMap::new(),
            channel_data: vec![],
        }    
    }

    pub(super) fn register_channel<C: Channel>(
        &mut self,
        config: ChannelConfiguration,
    ) -> ChannelId {
        // Check we don't overrun the channel ID
        if self.channel_data.len() >= (u32::MAX as usize) {
            panic!("Exceeded channel limit of {}", u32::MAX);
        }
        
        // Check the channel doesn't already exist
        let type_id = TypeId::of::<C>();
        #[cfg(feature="reflect")]
        let type_path = C::type_path();
        if self.channel_type_ids.get(&type_id).is_some() {
            #[cfg(feature="reflect")]
            panic!("A channel was registered twice: {type_path}");
            #[cfg(not(feature="reflect"))]
            panic!("A channel was registered twice: {type_id:?}");
        }

        // Add to map
        let channel_id = ChannelId::try_from(self.channel_count()).unwrap();
        self.channel_type_ids.insert(type_id, channel_id);
        
        #[cfg(feature="reflect")]
        self.channel_data.push(ChannelData {
            type_id,
            type_path,
            channel_id,

            entity_id: entity,
            config,
        });

        #[cfg(not(feature="reflect"))]
        self.channel_data.push(ChannelData {
            type_id,
            channel_id,

            config,
        });

        channel_id
    }

    /// Gets the id from the `ToChannelId` implementation.
    #[inline]
    pub fn channel_id(&self, value: impl ToChannelId) -> Option<ChannelId> {
        value.to_channel_id(self)
    }

    /// Returns a reference to the channel configuration.
    pub fn channel_config(&self, id: impl ToChannelId) -> Option<&ChannelData> {
        self.channel_data.get(Into::<usize>::into(id.to_channel_id(self)?))
    }

    /// Returns whether the channel exists.
    pub fn channel_exists(&self, id: ChannelId) -> bool {
        self.channel_data.len() >= Into::<usize>::into(id)
    }

    /// Returns how many channels currently exist.
    pub fn channel_count(&self) -> u32 {
        TryInto::<u32>::try_into(self.channel_data.len()).unwrap()
    }

    /// Returns an iterator of all existing channel ids.
    pub fn channel_ids(&self) -> impl Iterator<Item = ChannelId> {
        (0..self.channel_count()).into_iter()
        .map(|f| ChannelId::try_from(f).unwrap())
    }
}

impl Default for ChannelRegistryInner {
    fn default() -> Self {
        Self {
            channel_type_ids: BTreeMap::new(),
            channel_data: vec![],
        }
    }
}

/// Channel information generated when `add_channel` is run.
pub struct ChannelData {
    /// The channel's `TypeId`.
    pub type_id: TypeId,

    /// The channel's `TypePath` (from `bevy_reflect`)
    #[cfg(feature="reflect")]
    pub type_path: &'static str,

    /// The channel's sequential ID assigned by the registry.
    pub channel_id: ChannelId,

    /// The config of the channel.
    /// Since `ChannelData` implements `Deref` for `ChannelConfiguration`, this is just clutter.
    config: ChannelConfiguration,
}

impl std::ops::Deref for ChannelData {
    type Target = ChannelConfiguration;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}