use bevy_ecs::prelude::*;
use untrusted::{Input, Reader};
use crate::utils::IntegerFromByteSlice;

// This defines compatibilities between different versions of the crate
// It's different from the crate version since breaking changes in the crate
// may not necessarily be breaking changes in the network protocol
pub(crate) static TRANSPORT_VERSION_DATA: NetworkVersionData = NetworkVersionData {
    ident: 0x0, // If you're forking, make this a random value
    major: 0,   // If you're making a breaking change, increment this value
    minor: 0,   // if you're making a non-breaking change, increment this value
};

// The list of minor network-versions **of this crate** that should be blocked.
pub(crate) static BANNED_MINOR_VERSIONS: &[u32] = &[];

// TODO: This could be made part of Stardust's core library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NetworkVersionData {
    pub ident: u64,
    pub major: u32,
    pub minor: u32,
}

impl NetworkVersionData {
    pub(crate) fn placeholder() -> Self {
        Self {
            ident: 0x0,
            major: 0x0,
            minor: 0x0,
        }
    }

    pub(crate) fn from_bytes(bytes: [u8; 16]) -> NetworkVersionData {
        // Create reader object
        let mut reader = Reader::new(Input::from(&bytes));

        // Convert values
        let ident = u64::from_byte_slice(&mut reader).unwrap();
        let major = u32::from_byte_slice(&mut reader).unwrap();
        let minor = u32::from_byte_slice(&mut reader).unwrap();

        // Return value
        return Self { ident, major, minor };
    }

    pub(crate) fn to_bytes(&self) -> [u8; 16] {
        // Storage type
        let mut bytes = [0u8; 16];

        // Write values
        bytes[..8].copy_from_slice(&self.ident.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.major.to_be_bytes());
        bytes[12..16].copy_from_slice(&self.minor.to_be_bytes());

        // Return value
        return bytes;
    }
}

/// Network version information, distinct from your crate version.
/// 
/// Values must stay stable across compilations, platforms, and architectures.
/// The best way to do this is to hardcode a literal value and pass it during app setup.
#[derive(Debug, Clone)]
pub struct ApplicationNetworkVersion {
    /// A unique app identifier.
    /// Peers with different app identifiers cannot connect.
    pub ident: u64,

    /// The major network version.
    /// Peers with different major versions cannot connect.
    /// 
    /// For changes that *will* cause issues with older clients, increment this number.
    pub major: u32,

    /// The minor network version.
    /// 
    /// For changes that *won't* cause issues with older clients, increment this number.
    /// You should also reset this number every time you increment the `major` value.
    pub minor: u32,

    /// A list of minor network versions that are prohibited.
    /// Useful if you need to reject connections from yanked versions of your crate.
    pub banlist: &'static [u32],
}

impl ApplicationNetworkVersion {
    pub(crate) fn into_version(&self) -> NetworkVersionData {
        NetworkVersionData {
            ident: self.ident,
            major: self.major,
            minor: self.minor,
        }
    }
}

#[derive(Debug, Resource, Clone)]
pub(crate) struct AppNetVersionWrapper(pub ApplicationNetworkVersion);