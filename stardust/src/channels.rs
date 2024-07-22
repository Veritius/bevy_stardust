use std::{any::{Any, TypeId}, collections::BTreeMap, hash::Hash, marker::PhantomData, ops::{Deref, DerefMut}, sync::Arc};
use bevy::{ecs::{component::Tick, system::{SystemMeta, SystemParam}, world::unsafe_world_cell::UnsafeWorldCell}, prelude::*};

pub trait NetIdentifier: Any {}
impl<T> NetIdentifier for T where T: Any {}

/// A type of channel, used for distinguishing between registries.
pub trait ChannelType: 'static + Send + Sync {
    /// The configuration for the channel.
    type Config: 'static + Send + Sync + Hash;
}

/// A unique identifier for a channel, generated during application setup.
/// 
/// A `ChannelId` is used to identify a channel without type information,
/// such as in a transport layer or associative arrays where `TypeId`
/// would be excessive. Channel registration also ensures that the same
/// `ChannelId` refers to the same channel, regardless of compilation.
/// This only holds true if the [ordering constraints](super) are obeyed.
/// 
/// Note that channel identifiers are only unique to the
/// `World` belonging to the `App` they were registered to.
/// Using them in a different `World` or `App` may panic,
/// or have additional consequences because of transport
/// layers, such as causing undefined behavior. Stardust
/// will never cause UB by itself no matter how badly you
/// misuse it, but it's worth reading transport layer docs.
#[derive(Debug, Reflect)]
#[repr(transparent)]
pub struct ChannelId<R: ChannelType> {
    value: u32,
    #[reflect(ignore)]
    phantom: PhantomData<R>,
}

impl<R: ChannelType> From<u32> for ChannelId<R> {
    fn from(value: u32) -> Self {
        Self { value, phantom: PhantomData }
    }
}

impl<R: ChannelType> From<ChannelId<R>> for u32 {
    fn from(value: ChannelId<R>) -> Self {
        value.value
    }
}

impl<R: ChannelType> From<ChannelId<R>> for usize {
    fn from(value: ChannelId<R>) -> Self {
        value.value as usize
    }
}

impl<R: ChannelType> Clone for ChannelId<R> {
    fn clone(&self) -> Self {
        Self { value: self.value, phantom: self.phantom }
    }
}

impl<R: ChannelType> Copy for ChannelId<R> {}

impl<R: ChannelType> PartialEq for ChannelId<R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value.eq(&other.value)
    }
}

impl<R: ChannelType> Eq for ChannelId<R> {}

impl<R: ChannelType> PartialOrd for ChannelId<R> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl<R: ChannelType> Ord for ChannelId<R> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<R: ChannelType> Hash for ChannelId<R> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state)
    }
}

#[derive(Resource)]
pub(crate) struct RegistryBuilder<R: ChannelType> {
    registry: ChannelRegistry<R>,
}

impl<R: ChannelType> RegistryBuilder<R> {
    pub fn new() -> Self {
        Self {
            registry: ChannelRegistry::new(),
        }
    }

    pub fn finish(mut self) -> RegistryComplete<R> {
        self.items.shrink_to_fit();

        RegistryComplete {
            registry: Arc::new(self.registry),
        }
    }
}

impl<R: ChannelType> Deref for RegistryBuilder<R> {
    type Target = ChannelRegistry<R>;

    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

impl<R: ChannelType> DerefMut for RegistryBuilder<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.registry
    }
}

#[derive(Resource)]
pub(crate) struct RegistryComplete<R: ChannelType> {
    registry: Arc<ChannelRegistry<R>>,
}

impl<R: ChannelType> Deref for RegistryComplete<R> {
    type Target = ChannelRegistry<R>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.registry
    }
}

pub(crate) struct ChannelRegistry<R: ChannelType> {
    type_ids: BTreeMap<TypeId, ChannelId<R>>,
    items: Vec<Registration<R>>,
}

impl<R: ChannelType> ChannelRegistry<R> {
    fn new() -> Self {
        Self {
            type_ids: BTreeMap::new(),
            items: Vec::new(),
        }
    }

    pub(crate) fn register<I: NetIdentifier>(
        &mut self,
        config: R::Config,
    ) -> ChannelId<R> {
        // Check we don't overrun the channel ID
        if self.items.len() >= (u32::MAX as usize) {
            panic!("Exceeded channel limit of {}", u32::MAX);
        }
        
        // Check the registration doesn't already exist
        let type_id = TypeId::of::<I>();
        let type_name = std::any::type_name::<I>();
        if self.type_ids.get(&type_id).is_some() {
            panic!("A channel was registered twice: {}", std::any::type_name::<I>());
        }

        // Add to map
        let unique_id = ChannelId { value: self.count(), phantom: PhantomData };
        self.type_ids.insert(type_id, unique_id);
        
        self.items.push(Registration {
            metadata: Metadata {
                type_id,
                type_name,
                unique_id,
                _hidden: (),
            },

            config,
        });

        unique_id
    }

    pub(crate) fn id_from_type<T: NetIdentifier>(&self) -> Option<ChannelId<R>> {
        self.type_ids.get(&TypeId::of::<T>()).copied()
    }

    pub(super) fn get_registration(&self, id: ChannelId<R>) -> Option<&Registration<R>> {
        self.items
            .get(Into::<usize>::into(id))
    }

    /// Returns a reference to the channel metadata.
    pub fn metadata(&self, id: ChannelId<R>) -> Option<&Metadata<R>> {
        self.get_registration(id).map(|v| &v.metadata)
    }

    /// Returns a reference to the channel configuration.
    pub fn data(&self, id: ChannelId<R>) -> Option<&R::Config> {
        self.get_registration(id).map(|v| &v.config)
    }

    /// Returns whether the channel exists.
    pub fn exists(&self, id: ChannelId<R>) -> bool {
        self.items.len() >= Into::<usize>::into(id)
    }

    /// Returns how many channels currently exist.
    pub fn count(&self) -> u32 {
        TryInto::<u32>::try_into(self.items.len()).unwrap()
    }
}

pub struct Registration<R: ChannelType> {
    pub metadata: Metadata<R>,
    pub config: R::Config,
}

impl<R: ChannelType> Deref for Registration<R> {
    type Target = R::Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

/// Metadata about a channel, generated during channel registration.
pub struct Metadata<R: ChannelType> {
    /// The registration's `TypeId`.
    pub type_id: TypeId,

    /// The registration's type name, from the `Any` trait.
    /// This is only useful for debugging, and is not stable across compilation.
    pub type_name: &'static str,

    /// The registration's sequential ID assigned by the registry.
    pub unique_id: ChannelId<R>,

    _hidden: (),
}

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
pub(crate) struct Registrations<'w, R: ChannelType> {
    // This hides the ChannelRegistryFinished type so that it
    // cannot be removed from the World, which would be bad
    finished: Res<'w, RegistryComplete<R>>,
}

// impl<'w, R: Registrable> Channels<'w, R> {
//     /// Returns an `Arc` to the underlying `ChannelRegistry`.
//     /// This allows the registry to be used in asynchronous contexts.
//     pub fn clone_arc(&self) -> Arc<ChannelRegistry> {
//         self.finished.0.clone()
//     }
// }

impl<'a, R: ChannelType> AsRef<RegistryComplete<R>> for Registrations<'a, R> {
    #[inline]
    fn as_ref(&self) -> &RegistryComplete<R> {
        &self.finished
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
pub struct ChannelData<'a, R: ChannelType, I: NetIdentifier> {
    registration: &'a Registration<R>,
    phantom: PhantomData<I>,
}

impl<R: ChannelType, I: NetIdentifier> ChannelData<'_, R, I> {
    /// Returns the [`ChannelId`] assigned to `C`.
    #[inline]
    pub fn id(&self) -> ChannelId<R> {
        self.metadata().unique_id
    }

    /// Returns the [`ChannelMetadata`] of channel `C`.
    #[inline]
    pub fn metadata(&self) -> &Metadata<R> {
        &self.registration.metadata
    }

    /// Returns the [`ChannelConfiguration`] of channel `C`.
    #[inline]
    pub fn config(&self) -> &R::Config {
        &self.registration.config
    }
}

impl<'a, R: ChannelType, I: NetIdentifier> Clone for ChannelData<'a, R, I> {
    fn clone(&self) -> ChannelData<'a, R, I> {
        Self {
            registration: self.registration,
            phantom: PhantomData,
        }
    }
}

impl<'a, R: ChannelType, I: NetIdentifier> Copy for ChannelData<'a, R, I> {}

pub struct ChannelDataState<R: ChannelType> {
    // Directly use the State type from the SystemParam implementation
    // This avoids type errors if it's changed in future. It shouldn't, but eh.
    // The lifetime should be irrelevant here. If it isn't, a type error is thrown.
    res_state: <Res<'static, RegistryComplete<R>> as SystemParam>::State,
    channel: ChannelId<R>,
}

unsafe impl<R, I> SystemParam for ChannelData<'_, R, I>
where
    R: ChannelType,
    I: NetIdentifier,
{
    type State = ChannelDataState<R>;
    type Item<'world, 'state> = ChannelData<'world, R, I>;

    fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State {
        let res_state = <Res<RegistryComplete<R>> as SystemParam>::init_state(world, system_meta);
        let registry = world.resource::<RegistryComplete<R>>();
        let channel = registry.id_from_type::<I>().unwrap();
        return ChannelDataState { res_state, channel };
    }

    unsafe fn get_param<'world, 'state>(
        state: &'state mut Self::State,
        system_meta: &SystemMeta,
        world: UnsafeWorldCell<'world>,
        change_tick: Tick,
    ) -> Self::Item<'world, 'state> {
        let registry = <Res<'world, RegistryComplete<R>> as SystemParam>::get_param(
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

pub(crate) fn build<R: ChannelType>(app: &mut App) {
    app.insert_resource(RegistryBuilder::<R>::new());
}

pub(crate) fn finish<R: ChannelType>(app: &mut App) {
    let builder = app.world_mut().remove_resource::<RegistryBuilder<R>>().unwrap();
    app.insert_resource(builder.finish());
}