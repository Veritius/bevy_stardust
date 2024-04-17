use bevy::prelude::*;

/// How 'secure' a connection is.
/// This is set by the transport layer that controls the connection.
/// See variant documentation for specific information.
/// 
/// This type implements `Ord`, with 'greater' orderings corresponding to better security.
///
/// This value is set by the transport layer managing this peer.
/// It's up to it to provide an appropriate value here.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[reflect(Debug, Component, PartialEq)]
#[non_exhaustive]
pub enum NetworkSecurity {
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
