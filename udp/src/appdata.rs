use unbytes::{EndOfInput, Reader};

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
    pub(crate) fn from_bytes(reader: &mut Reader) -> Result<NetworkVersionData, EndOfInput> {
        Ok(Self {
            ident: u64::from_be_bytes(reader.read_array::<8>()?),
            major: u32::from_be_bytes(reader.read_array::<4>()?),
            minor: u32::from_be_bytes(reader.read_array::<4>()?),
        })
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

#[test]
fn to_from_bytes_test() {
    use bytes::Bytes;
    use unbytes::*;

    let original = NetworkVersionData {
        ident: 48512967252744321,
        major: 2481257245,
        minor: 2528142859,
    };

    let bytes = original.to_bytes();
    let bytes = Bytes::copy_from_slice(&bytes[..]);

    let mut reader = Reader::new(bytes);
    let parsed = NetworkVersionData::from_bytes(&mut reader).unwrap();

    assert_eq!(original.ident, parsed.ident);
    assert_eq!(original.major, parsed.major);
    assert_eq!(original.minor, parsed.minor);
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
    pub(crate) fn as_nvd(&self) -> NetworkVersionData {
        NetworkVersionData {
            ident: self.ident,
            major: self.major,
            minor: self.minor,
        }
    }
}