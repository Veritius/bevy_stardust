// Variable length integer implementation based on RFC 9000 (QUIC)

use std::fmt::Debug;
use bytes::BufMut;
use unbytes::{EndOfInput, Reader};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct VarInt(u64);

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
        return Ok(Self(value))
    }
}

impl TryFrom<usize> for VarInt {
    type Error = ();

    #[inline]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        // The pointer size is smaller than the maximum value on a 16 or 32 bit system
        // which means this conversion won't ever cause any problems.
        #[cfg(any(target_pointer_width="16", target_pointer_width="32"))]
        return Ok(Self(value as u64));

        // On 64-bit targets, we actually have to check.
        #[cfg(target_pointer_width="64")]
        return (value as u64).try_into()
    }
}

impl From<VarInt> for u64 {
    #[inline]
    fn from(value: VarInt) -> Self {
        value.0
    }
}

impl From<VarInt> for usize {
    #[inline]
    fn from(value: VarInt) -> Self {
        value.0 as usize
    }
}

impl Debug for VarInt {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl VarInt {
    pub const MAX: u64 = 2u64.pow(62);

    pub const B1_LMT: u64 = 2u64.pow(6);
    pub const B2_LMT: u64 = 2u64.pow(14);
    pub const B4_LMT: u64 = 2u64.pow(30);
    pub const B8_LMT: u64 = 2u64.pow(62);

    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        const MASK: u8 = 0b0000_0011;

        let mut bytes = [0u8; 8];
        let fb = reader.read_byte()?;
        bytes[0] = fb & !MASK;

        match fb & MASK {
            0b00 => {},
            0b01 => {
                bytes[1] = reader.read_byte()?;
            },
            0b10 => {
                let slc = reader.read_slice(3)?;
                bytes[1..4].copy_from_slice(slc);
            },
            0b11 => {
                let slc = reader.read_slice(7)?;
                bytes[1..8].copy_from_slice(slc);
            },
            _ => unreachable!(),
        }

        let val = u64::from_le_bytes(bytes) >> 2;
        return Ok(Self(val))
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) {
        // Little endian bytes: least significant bits first
        let mut b = (self.0 << 2).to_le_bytes();

        match self.0 {
            // First case: one byte
            x if x < Self::B1_LMT => {
                buf.put_u8(b[0])
            },

            // Second case: two bytes
            x if x < Self::B2_LMT => {
                b[0] |= 0b01;
                buf.put(&b[..2]);
            },

            // Third case: four bytes
            x if x < Self::B4_LMT => {
                b[0] |= 0b10;
                buf.put(&b[..4]);
            },

            // Fourth case: eight bytes
            x if x < Self::B8_LMT => {
                b[0] |= 0b11;
                buf.put(&b[..8]);
            },

            _ => unreachable!("bad varint"),
        }
    }

    /// Returns how many bytes this varint will use on the wire.
    pub fn size(&self) -> usize {
        match self.0 {
            x if x < Self::B1_LMT => 1,
            x if x < Self::B2_LMT => 2,
            x if x < Self::B4_LMT => 4,
            x if x < Self::B8_LMT => 8,
            _ => unreachable!("bad varint"),
        }
    }
}

#[test]
fn back_and_forth_test() {
    use bytes::*;
    use unbytes::*;

    fn serial_test(value: u64) {
        let value = VarInt::try_from(value)
            .expect("Value passed to serial_test was not representable in a varint");

        let mut bytes = BytesMut::with_capacity(8);
        value.write(&mut bytes);

        let bytes = bytes.freeze();
        let mut reader = Reader::new(bytes);
        assert_eq!(reader.remaining(), value.size());

        let new = VarInt::read(&mut reader).unwrap();
        assert_eq!(value, new);
    }

    static TEST_SET: &[u64] = &[
        0, 1, 2, 4, 8, 16, 32, 63, 64, 65, 66, // 0b00
        8000, 10000, 16000, 16383, 16384, 16385, // 0b01
        107374000, 1073741823, 1073741824, 1073741825, // 0b10
        4611686017999999999, 4611686018000000000, 4611686018000000001, // 0b11
    ];

    for item in TEST_SET {
        serial_test(*item);
    }
}