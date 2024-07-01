use std::fmt::Debug;
use bevy::reflect::Reflect;

/// The direction a message is going, as an enum for dynamic use.
/// 
/// For use in the type system, see the [`MessageDirection`] trait.
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
/// This is a [sealed trait] implemented by two [empty enums], [`Outgoing`] and [`Incoming`].
/// These are intended to be used exclusively within the type system, such as on [`PeerMessages<D>`].
/// This allows types to have 'directional' variations to indicate their role in message flow.
/// 
/// The enum [`NetDirection`] is useful to carry this information at runtime.
/// It is also returned by any `MessageDirection` implementor with the `net_dir` function.
/// 
/// [sealed trait]: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
/// [empty enums]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#empty-types
/// [`PeerMessages<D>`]: crate::connections::PeerMessages
pub trait MessageDirection: Debug + Send + Sync + Reflect + sealed::Sealed {
    /// Returns the corresponding [`NetDirection`].
    fn net_dir() -> NetDirection;
}

/// Messages being transmitted from this peer to a remote peer.
/// 
/// Counterpart to [`Incoming`], and corresponds to [`NetDirection::Incoming`].
/// 
/// This type **cannot** be instantiated and is only intended for use in the type system.
/// For more information on message direction, see the [`MessageDirection`] trait.
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub enum Outgoing {}

impl MessageDirection for Outgoing {
    fn net_dir() -> NetDirection {
        NetDirection::Outgoing
    }
}

/// Messages being transmitted to this peer by a remote peer.
/// 
/// Counterpart to [`Outgoing`], and corresponds to [`NetDirection::Outgoing`].
/// 
/// This type **cannot** be instantiated and is only intended for use in the type system.
/// For more information on message direction, see the [`MessageDirection`] trait.
#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Debug)]
pub enum Incoming {}

impl MessageDirection for Incoming {
    fn net_dir() -> NetDirection {
        NetDirection::Incoming
    }
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Outgoing {}
    impl Sealed for super::Incoming {}
}