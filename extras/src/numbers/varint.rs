use std::fmt::{Debug, Display};
use bevy_stardust::messages::bytes::{Buf, BufMut};
use bevy_stardust::prelude::*;

/// A variable length integer that can store values up to `(2^62)-1`.
/// 
/// Based on [RFC 9000 Section 16](https://www.rfc-editor.org/rfc/rfc9000.html#name-variable-length-integer-enc) (Variable-Length Integer Encoding).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VarInt(u64);

impl VarInt {
    /// The maximum representable value of a `VarInt`.
    pub const MAX: u64 = 2u64.pow(62) - 1;

    /// Creates a `VarInt` from a `u32`.
    /// As this function cannot fail, it is usable in const contexts.
    #[inline]
    pub const fn from_u32(value: u32) -> Self {
        Self(value as u64)
    }

    /// Decodes a `VarInt` from a [`Buf`].
    pub fn read<B: Buf>(b: &mut B) -> Result<Self, ()> {
        const MASK: u8 = 0b0000_0011;

        // Check there's anything left in the buffer
        if b.remaining() < 1 { return Err(()) }

        let mut bytes = [0u8; 8];
        let first = b.get_u8();
        bytes[0] = first & !MASK;

        match first & MASK {
            0b00 => {},

            0b01 => {
                if b.remaining() < 1 { return Err(()) }
                bytes[1] = b.get_u8();
            },

            0b10 => {
                if b.remaining() < 3 { return Err(()); }
                b.copy_to_slice(&mut bytes[1..4]);
            },

            0b11 => {
                if b.remaining() < 7 { return Err(()); }
                b.copy_to_slice(&mut bytes[1..8]);
            },

            _ => unreachable!(),
        }

        // The result has to be bitshifted by 2
        // due to the length header
        return Ok(Self(u64::from_le_bytes(bytes) >> 2));
    }

    /// Encodes a `VarInt` to a [`BufMut`].
    pub fn write<B: BufMut>(&self, b: &mut B) -> Result<(), ()> {
        let mut bytes = (self.0 << 2).to_le_bytes();
        let len = self.len();
        if len as usize > b.remaining_mut() { return Err(()); }

        match len {
            1 => {
                b.put_u8(bytes[0]);
            },

            2 => {
                bytes[0] |= 0b01;
                b.put(&bytes[..2]);
            },

            4 => {
                bytes[0] |= 0b10;
                b.put(&bytes[..4]);
            },

            8 => {
                bytes[0] |= 0b11;
                b.put(&bytes[..8]);
            },

            _ => unreachable!(),
        }

        return Ok(());
    }

    /// Returns the amount of bytes that would be written if `write` were used.
    pub fn len(&self) -> u8 {
        // SAFETY: A VarInt that would return an Err cannot be created, so this case cannot occur.
        unsafe { Self::len_u64(self.0).unwrap_unchecked() }
    }

    /// Estimate the encoded size of a `VarInt` with this value.
    pub fn len_u32(value: u32) -> u8 {
        let x = value;
        if x <= 63                  { return 1; }
        if x <= 16383               { return 2; }
        if x <= 1073741823          { return 4; }
        return 8;
    }

    /// Estimate the encoded size of a `VarInt` with this value.
    /// 
    /// Since `u64` can represent values a `VarInt` can't, this function can fail.
    pub fn len_u64(value: u64) -> Result<u8, ()> {
        if value <= 63                  { return Ok(1); }
        if value <= 16383               { return Ok(2); }
        if value <= 1073741823          { return Ok(4); }
        if value <= 4611686018427387903 { return Ok(8); }
        return Err(());
    }
}

impl From<u32> for VarInt {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value as u64)
    }
}

impl TryFrom<u64> for VarInt {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > Self::MAX { return Err(()); }
        return Ok(Self(value));
    }
}

impl TryFrom<usize> for VarInt {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        #[cfg(target_pointer_width="32")]
        return Ok(Self(value as u64));

        #[cfg(target_pointer_width="64")]
        (value as u64).try_into()
    }
}

impl TryFrom<VarInt> for u32 {
    type Error = ();

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        if value.0 > u32::MAX as u64 { return Err(()); }
        return Ok(value.0 as u32);
    }
}

impl From<VarInt> for u64 {
    #[inline]
    fn from(value: VarInt) -> Self {
        value.0
    }
}

impl From<VarInt> for ChannelId {
    #[inline]
    fn from(value: VarInt) -> Self {
        value.into()
    }
}

impl From<ChannelId> for VarInt {
    #[inline]
    fn from(value: ChannelId) -> Self {
        VarInt::from_u32(value.into())
    }
}

impl Debug for VarInt {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for VarInt {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[test]
fn varint_encoding() {
    use std::io::Cursor;

    static TEST_SET: &[u64] = &[
        0, 1, 2, 4, 8, 16, 32, 63, 64, 65, 66,
        8000, 10000, 16000, 16383, 16384, 16385,
        107374000, 1073741823, 1073741824, 1073741825,
        4611686017999999999, 4611686018000000000, 4611686018000000001,
        4611686018427387901, 4611686018427387902, 4611686018427387903,
    ];

    fn serial_test(value: u64, bytes: &mut Vec<u8>) {
        let value = VarInt::try_from(value)
            .expect("Value passed to serial_test was not representable in a varint");

        // Serialise
        value.write(bytes).unwrap();
        assert_eq!(bytes.len(), value.len() as usize);

        // Deserialise
        let mut cursor = Cursor::new(&bytes[..]);
        let new = VarInt::read(&mut cursor).unwrap();
        assert_eq!(value, new);
    }

    let mut bytes: Vec<u8> = Vec::with_capacity(8);
    for value in TEST_SET {
        serial_test(*value, &mut bytes);
        bytes.clear();
    }
}

#[cfg(feature="octs")]
mod octs {
    use super::VarInt;
    use octs::{Encode, EncodeLen, FixedEncodeLenHint, Decode};

    impl Encode for VarInt {
        type Error = ();

        fn encode(&self, mut dst: impl octs::Write) -> Result<(), octs::BufTooShortOr<Self::Error>> {
            self.write(&mut dst).map_err(|_| octs::BufTooShortOr::TooShort)
        }
    }

    impl EncodeLen for VarInt {
        fn encode_len(&self) -> usize {
            self.len() as usize
        }
    }

    impl FixedEncodeLenHint for VarInt {
        const MAX_ENCODE_LEN: usize = 8;
        const MIN_ENCODE_LEN: usize = 1;
    }

    impl Decode for VarInt {
        type Error = ();

        fn decode(mut src: impl octs::Read) -> Result<Self, octs::BufTooShortOr<Self::Error>> {
            VarInt::read(&mut src).map_err(|_| octs::BufTooShortOr::TooShort)
        }
    }
}