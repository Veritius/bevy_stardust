use bevy::prelude::*;
use serde::{Serialize, Deserialize};

pub use inner::Replicable;

/// Trait for resources that can be replicated.
/// Automatically implemented for types that satisfy the requirements.
pub trait ReplicableResource: Resource + Replicable {}
impl<T> ReplicableResource for T where T: Resource + Replicable {}

/// Trait for components that can be replicated.
/// Automatically implemented for types that satisfy the requirements.
pub trait ReplicableComponent: Component + Replicable {}
impl<T> ReplicableComponent for T where T: Component + Replicable {}

/// Trait for events that can be replicated as-is.
/// Automatically implemented for types that satisfy the requirements.
pub trait ReplicableEvent: Event + Replicable {}
impl<T> ReplicableEvent for T where T: Event + Replicable {}

#[cfg(not(feature="reflect"))]
mod inner {
    use super::*;

    /// Trait for types that can be replicated.
    /// Automatically implemented for types that satisfy the requirements.
    pub trait Replicable: Serialize + for<'a> Deserialize<'a> {}
    impl<T> Replicable for T where T: Serialize + for<'a> Deserialize<'a> {}   
}

#[cfg(feature="reflect")]
mod inner {
    use super::*;
    use bevy::{prelude::*, reflect::GetTypeRegistration};

    /// Trait for types that can be replicated.
    /// Automatically implemented for types that satisfy the requirements.
    pub trait Replicable: Reflect + TypePath + GetTypeRegistration + Serialize + for<'a> Deserialize<'a> {}
    impl<T> Replicable for T where T: Reflect + TypePath + GetTypeRegistration + Serialize + for<'a> Deserialize<'a> {}   
}