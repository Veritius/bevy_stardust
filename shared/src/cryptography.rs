use std::{net::SocketAddr, fmt::Debug};
use bevy::prelude::{Resource, App};
use rand::Rng;

/// Shared authentication server. Securely connects the client and server to perform a key exchange.
#[derive(Debug, Resource)]
pub(crate) struct AuthServerLocation {
    address: SocketAddr,
}

/// The **unique** private key of this peer. Used for signing and encryption.
#[derive(Resource)]
pub struct CryptoPrivateKey {
    private_key: [u8; 32],
}

impl CryptoPrivateKey {
    pub fn set_private_key(&mut self, key: [u8; 32]) {
        self.private_key = key;
    }

    /// Regenerates the private key, filling it with random data.
    /// Uses system random, so probably isn't entirely secure.
    pub fn regenerate_key(&mut self) {
        let mut random = rand::thread_rng();
        random.fill(&mut self.private_key);
    }
}

impl Debug for CryptoPrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CryptoPrivateKey")
        .field("private_key", &"[hidden]")
        .finish()
    }
}

pub trait NetworkCryptographyAppExt {
    /// Sets the shared cryptographic authority used in the key exchange stage of authentication to prevent MITM attacks.
    fn set_auth_server(&mut self, address: impl Into<SocketAddr>);
    /// Sets the private key of the client or server. Used for signing/encryption.
    fn set_private_key(&mut self, private_key: [u8; 32]);
}

impl NetworkCryptographyAppExt for App {
    fn set_auth_server(&mut self, address: impl Into<SocketAddr>) {
        self.insert_resource(AuthServerLocation {
            address: address.into(),
        });
    }

    fn set_private_key(&mut self, private_key: [u8; 32]) {
        self.insert_resource(CryptoPrivateKey {
            private_key,
        });
    }
}