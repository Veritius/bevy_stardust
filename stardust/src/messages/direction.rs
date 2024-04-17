use std::fmt::Debug;
use bevy::reflect::Reflect;

/// The direction a message is going, as an enum for dynamic use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum Direction {
    /// Messages being sent to a remote peer.
    Outgoing,
    /// Messages being received from a remote peer.
    Incoming,
}

/// The direction a message is going, as a trait for use in the type system.
/// 
/// Implemented by:
/// - [`Outgoing`], corresponding to [`Direction::Outgoing`]
/// - [`Incoming`], corresponding to [`Direction::Incoming`]
pub trait DirectionType: Debug + Send + Sync + Reflect + sealed::Sealed {
    /// Returns the corresponding [`Direction`].
    fn as_enum() -> Direction;
}

/// Messages being sent to a remote peer. Counterpart to [`Incoming`].
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub struct Outgoing;
impl DirectionType for Outgoing {
    fn as_enum() -> Direction {
        Direction::Outgoing
    }
}

/// Messages being received from a remote peer. Counterpart to [`Outgoing`].
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub struct Incoming;
impl DirectionType for Incoming {
    fn as_enum() -> Direction {
        Direction::Incoming
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Outgoing {}
    impl Sealed for super::Incoming {}
}