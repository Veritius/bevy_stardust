use serde::{Serialize, Deserialize};

pub use inner::Replicable;

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