use serde::{Serialize, Deserialize};

/// Trait for types that can be replicated.
/// Automatically implemented for types that satisfy the requirements.
pub trait Replicable: Serialize + for<'a> Deserialize<'a> {}
impl<T> Replicable for T where T: Serialize + for<'a> Deserialize<'a> {}