use unbytes::{EndOfInput, Reader};

// This defines compatibilities between different versions of the crate
// It's different from the crate version since breaking changes in the crate
// may not necessarily be breaking changes in the network protocol
pub(crate) static TRANSPORT_VERSION_DATA: AppVersion = AppVersion {
    ident: 0x0, // If you're forking, make this a random value
    major: 0,   // If you're making a breaking change, increment this value
    minor: 0,   // if you're making a non-breaking change, increment this value
};

// The list of minor network-versions **of this crate** that should be blocked.
pub(crate) static BANNED_MINOR_VERSIONS: DeniedMinorVersions = &[];

/// The version of the application, shown over the network.
/// This is different from the crate's SemVer version value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppVersion {
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
}

/// Minor versions that will not be permitted to join.
pub type DeniedMinorVersions = &'static [u32];

impl AppVersion {
    pub(crate) fn from_bytes(reader: &mut Reader) -> Result<AppVersion, EndOfInput> {
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

    pub fn compare(
        &self,
        other: &Self,
        banlist: DeniedMinorVersions,
    ) -> Result<(), IncompatibilityReason> {
        use IncompatibilityReason::*;
        if self.ident != other.ident { return Err(MismatchedIdentifier); }
        if self.major != other.major { return Err(MismatchedMajorVersion); }
        if banlist.contains(&other.minor) { return Err(DeniedMinorVersion); }
        return Ok(());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncompatibilityReason {
    MismatchedIdentifier,
    MismatchedMajorVersion,
    DeniedMinorVersion,
}

#[test]
fn to_from_bytes_test() {
    use bytes::Bytes;
    use unbytes::*;

    let original = AppVersion {
        ident: 48512967252744321,
        major: 2481257245,
        minor: 2528142859,
    };

    let bytes = original.to_bytes();
    let bytes = Bytes::copy_from_slice(&bytes[..]);

    let mut reader = Reader::new(bytes);
    let parsed = AppVersion::from_bytes(&mut reader).unwrap();

    assert_eq!(original.ident, parsed.ident);
    assert_eq!(original.major, parsed.major);
    assert_eq!(original.minor, parsed.minor);
}