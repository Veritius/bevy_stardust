//! Serialisation and deserialisation functionality.

use bevy_stardust::prelude::*;

/// A function used to serialise `T` into a new [`Bytes`].
pub type SerialiseFn<T> = fn(&T) -> anyhow::Result<Bytes>;

/// A function used to deserialise `T` from a set of bytes.
pub type DeserialiseFn<T> = fn(Bytes) -> anyhow::Result<T>;

/// A set of common serialisation functions.
#[allow(missing_docs)]
#[derive(Clone)]
pub struct SerialisationFns<T> {
    pub serialise: SerialiseFn<T>,
    pub deserialise: DeserialiseFn<T>,
}