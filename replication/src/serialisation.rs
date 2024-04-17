//! Serialisation related functionality.

pub use bevy_stardust::messages::Bytes;

use anyhow::Result;

/// Function pointers for serialisation and deserialisation for `T`.
/// 
/// For a given `T`, the output of `serialise` put into `deserialise` must result in an identical `T`.
pub struct SerialisationFunctions<T> {
    /// Function to serialise `T`.
    pub serialise: fn(&T) -> Result<Bytes>,
    /// Function to deserialise `T` from a `Bytes`.
    pub deserialise: fn(Bytes) -> Result<T>,
}

// T doesn't need to be Clone since these are pointers
impl<T> Clone for SerialisationFunctions<T> {
    fn clone(&self) -> Self {
        Self {
            serialise: self.serialise,
            deserialise: self.deserialise,
        }
    }
}

impl<T> Copy for SerialisationFunctions<T> {}

#[cfg(feature="serde")]
mod serde_impls {
    use serde::{Serialize, Deserialize};
    use super::*;

    impl<T: Serialize + for<'a> Deserialize<'a>> SerialisationFunctions<T> {
        /// Create `SerialisationFunctions<T>` based on `T`'s implementation of [`Serialize`] and [`Deserialize`].
        pub fn serde() -> SerialisationFunctions<T> {
            Self {
                serialise: |t| match bincode::serialize(t) {
                    Ok(vec) => Ok(vec.into()),
                    Err(err) => Err(err.into()),
                },
                deserialise: |bytes| bincode::deserialize::<T>(&bytes).map_err(|e| e.into()),
            }
        }
    }
}