use std::fmt::Debug;
use bevy::reflect::Reflect;

/// The direction a message is going, as an enum for dynamic use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
pub enum NetDirection {
    /// Messages being transmitted from this peer to a remote peer.
    /// Corresponds to, and is returned by, [`Outgoing`].
    Outgoing,

    /// Messages being transmitted to this peer by a remote peer.
    /// Corresponds to, and is returned by, [`Incoming`].
    Incoming,
}

/// The direction a message is going, as a trait for use in the type system.
/// 
/// Implemented by:
/// - [`Outgoing`], corresponding to [`NetDirection::Outgoing`]
/// - [`Incoming`], corresponding to [`NetDirection::Incoming`]
pub trait NetDirectionType: Debug + Send + Sync + Reflect + sealed::Sealed {
    /// Returns the corresponding [`NetDirection`].
    fn net_dir() -> NetDirection;
}

/// Messages being transmitted from this peer to a remote peer.
/// Counterpart to [`Incoming`], and corresponds to [`NetDirection::Incoming`].
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub struct Outgoing;
impl NetDirectionType for Outgoing {
    fn net_dir() -> NetDirection {
        NetDirection::Outgoing
    }
}

/// Messages being transmitted to this peer by a remote peer.
/// Counterpart to [`Outgoing`], and corresponds to [`NetDirection::Outgoing`].
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub struct Incoming;
impl NetDirectionType for Incoming {
    fn net_dir() -> NetDirection {
        NetDirection::Incoming
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Outgoing {}
    impl Sealed for super::Incoming {}
}