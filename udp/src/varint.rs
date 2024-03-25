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

impl From<VarInt> for u64 {
    #[inline]
    fn from(value: VarInt) -> Self {
        value.0
    }
}

impl Debug for VarInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0 & (u64::MAX >> 2)).fmt(f)
    }
}

impl VarInt {
    pub const MAX: u64 = 2u64.pow(62);

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
        let x = self.0;
        let mut b = (self.0 << 2).to_le_bytes();

        if x < 2u64.pow(6) {
            buf.put_u8(b[0]);
        } else if x < 2u64.pow(14) {
            b[0] |= 0b01;
            buf.put(&b[..2]);
        } else if x < 2u64.pow(30) {
            b[0] |= 0b10;
            buf.put(&b[..4]);
        } else if x < 2u64.pow(62) {
            b[0] |= 0b11;
            buf.put(&b[..8]);
        } else {
            unreachable!("bad varint");
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