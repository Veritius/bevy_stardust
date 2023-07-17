use std::net::IpAddr;
use bevy::prelude::{Resource, App};
use rand::Rng;

/// Shared cryptographic authority. Used during key exchange to prevent MITM attacks.
#[derive(Debug, Resource)]
pub(crate) struct CryptoSharedAuthority {
    address: IpAddr,
    public_key: [u8; 16],
}

/// The **unique** private key of this peer. Used for signing and encryption.
#[derive(Debug, Resource)]
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

pub trait NetworkCryptographyAppExt {
    /// Sets the shared cryptographic authority used in the key exchange stage of authentication to prevent MITM attacks.
    fn set_cryptographic_authority(&mut self, address: impl Into<IpAddr>, public_key: [u8; 16]);
    /// Sets the private key of the client or server. Used for signing/encryption.
    fn set_private_key(&mut self, private_key: [u8; 32]);
}

impl NetworkCryptographyAppExt for App {
    fn set_cryptographic_authority(&mut self, address: impl Into<IpAddr>, public_key: [u8; 16]) {
        self.insert_resource(CryptoSharedAuthority {
            address: address.into(),
            public_key,
        });
    }

    fn set_private_key(&mut self, private_key: [u8; 32]) {
        self.insert_resource(CryptoPrivateKey {
            private_key,
        });
    }
}