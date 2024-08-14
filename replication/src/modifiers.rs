//! 'Modifiers' that can be attached to replicated components and resources to change their behavior.

use bevy::{ecs::component::StorageType, prelude::*};

/// Overrides the state of `T` that is replicated to other peers,
/// while keeping a hidden local state. Only effective if
/// this application is the authority over the target.
/// 
/// This is a tool to *intentionally* report incorrect game state to peers.
/// `Override` is very niche and should be used carefully, as it can cause
/// various hard-to-debug problems, such as with prediction.
pub struct Override<T> {
    /// The inner value.
    pub inner: T,
}

impl<T: Component> Component for Override<T> {
    const STORAGE_TYPE: StorageType = T::STORAGE_TYPE;
}

impl<T: Resource> Resource for Override<T> {}