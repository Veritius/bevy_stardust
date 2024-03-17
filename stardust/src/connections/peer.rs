//! "Peers" aka other computers over the network.

use std::time::Instant;
use bevy_ecs::prelude::*;

/// Another peer that this peer is aware of, representing someone else over the Internet.
/// 
/// - If you're writing server-side code, you can think of this as a client.
/// - If you're writing client-side code, you can think of this as the server.
/// - If you're writing peer-to-peer code, you can think of this as another peer in the mesh.
/// 
/// `NetworkPeer`s don't have any associated transport layer information by themselves.
/// However, they are always treated as an entity that stores information related to the network.
/// You shouldn't create, mutate, or delete this component unless you know what you're doing.
/// Managing these entities should be (and usually is) done by the transport layer.
/// 
/// Entities with `NetworkPeer` have their entity IDs used in the writing and reading APIs.
/// They are used as the 'target' of messages, and the transport layer will handle the actual sending and receiving.
#[derive(Debug, Component)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect))]
pub struct NetworkPeer {
    /// The point in time this peer was added to the `World`.
    pub joined: Instant,

    /// A unique UUID, if it has one.
    /// This can be used to identify a peer across network sessions.
    #[cfg(feature="uuids")]
    pub uuid: Option<uuid::Uuid>,

    /// The quality of the connection, from `0.0` to `1.0`.
    /// This is subjective and defined by the transport layer.
    /// `None` means a value is not provided.
    pub quality: Option<f32>,

    /// Round-trip time, in milliseconds.
    pub ping: u32,

    /// How secure the connection to this peer is.
    /// Set to [`Unprotected`](PeerSecurity::Unprotected) by default when using `NetworkPeer::new()`.
    pub security: PeerSecurity,

    disconnect_requested: bool,
}

impl NetworkPeer {
    /// Creates the component in the `Connecting` state.
    pub fn new() -> Self {
        Self {
            joined: Instant::now(),
            #[cfg(feature="uuids")]
            uuid: None,
            quality: None,
            ping: 0,
            security: PeerSecurity::Unauthenticated,
            disconnect_requested: false,
        }
    }

    /// Signals to the transport layer to disconnect the peer.
    /// This operation cannot be undone.
    pub fn disconnect(&mut self) {
        self.disconnect_requested = true
    }

    /// Returns `true` if [`disconnect`] has been used.
    /// This is intended for use by transport layers, and you should use [`NetworkPeerState`] instead.
    pub fn disconnect_requested(&self) -> bool {
        self.disconnect_requested
    }
}

/// How 'secure' a connection is.
/// This is set by the transport layer that controls the connection.
/// See variant documentation for specific information.
/// 
/// This type implements `Ord`, with 'greater' orderings corresponding to better security.
///
/// This value is set by the transport layer managing this peer.
/// It's up to it to provide an appropriate value here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect))]
pub enum PeerSecurity {
    /// Communication is encrypted but not authenticated, or is fully plain text.
    /// 
    /// **For end users:**
    /// This kind of connection should not be used for anything that must remain secret or private.
    /// It is vulnerable to [man in the middle attacks] like reading and modifying in-flight information.
    /// 
    /// [man in the middle attacks]: https://en.wikipedia.org/wiki/Man-in-the-middle_attack
    Unauthenticated,

    /// Communication is both encrypted and authenticated.
    ///
    /// **For end users:**
    /// - Encrypted traffic cannot be viewed by a man in the middle at any point once the handshake finishes.
    /// - You can exchange private information with the client in as much confidence as you have in your transport layers.
    ///
    /// Note that these guarantees are only valid if your transport layers are well implemented and use secure cryptography methods.
    /// Keep any cryptography-implementing transport layers up to date as much as possible, and use good judgement.
    /// 
    /// Additionally, since transport layers can read any and all outgoing messages, it's up to you to verify that they're safe.
    /// Regardless, it's not a good idea to transfer something like credit card details in the first place without incredible precautions.
    /// Some things (like banking. especially banking) should be left up to the experts.
    ///
    /// **For transport layer implementors:**
    /// - For TLS, this should be set if a full chain of trust is set up.
    ///    - Only TLS versions > 1.2 are acceptable (1.3 onward).
    ///    - You should always use the latest version of TLS. There's not really a reason not to.
    /// - Broken or flawed cryptography methods are not suitable for this variant. Broken cryptography is as bad as no cryptography.
    /// - If in doubt, *pick a lower level.*
    /// 
    /// **Examples of authenticated connections:**
    /// - [Pre-shared keys](https://en.wikipedia.org/wiki/Pre-shared_key)
    /// - [Transport Layer Security](https://en.wikipedia.org/wiki/Transport_Layer_Security)
    /// - [netcode.io](https://github.com/networkprotocol/netcode.io/blob/master/STANDARD.md)
    Authenticated,
}