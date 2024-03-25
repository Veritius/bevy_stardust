// Variable length integer implementation based on RFC 9000 (QUIC)
// Some code taken from quinn-proto, licensed under the MIT License
// https://github.com/quinn-rs/quinn/blob/a2a214b968fbcbc9aa66aba4393851b3d6ab5b49/quinn-proto/src/varint.rs
// Full copy of the license is here: https://github.com/quinn-rs/quinn/blob/a2a214b968fbcbc9aa66aba4393851b3d6ab5b49/LICENSE-MIT

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
    type Error = u64;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > 2u64.pow(62) { return Err(value); }
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
        (self.0 & u64::MAX >> 2).fmt(f)
    }
}

impl VarInt {
    pub const MAX: Self = Self((1 << 62) - 1);
    pub const MIN: Self = Self(0);

    pub fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        let mut bytes = [0u8; 8];
        let tag = reader.read_byte()?;
        bytes[0] = tag & 0b0011_1111;

        match tag >> 6 {
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

        return Ok(Self(u64::from_be_bytes(bytes)))
    }

    pub fn write<B: BufMut>(&self, buf: &mut B) {
        let x = self.0;
        if x < 2u64.pow(6) {
            buf.put_u8(x as u8);
        } else if x < 2u64.pow(14) {
            buf.put_u16(0b01 << 14 | x as u16);
        } else if x < 2u64.pow(30) {
            buf.put_u32(0b10 << 30 | x as u32);
        } else if x < 2u64.pow(62) {
            buf.put_u64(0b11 << 62 | x);
        } else {
            unreachable!("bad varint");
        }
    }
}

#[test]
fn back_and_forth_test() {
    use bytes::*;
    use unbytes::*;

    fn serial_test(value: VarInt) {
        let mut bytes = BytesMut::with_capacity(8);
        value.write(&mut bytes);

        let mut reader = Reader::new(bytes.freeze());
        let new = VarInt::read(&mut reader).unwrap();

        assert_eq!(value, new);
    }

    serial_test(VarInt(0));
    serial_test(VarInt(1));
    serial_test(VarInt(7));
    serial_test(VarInt(50));
    serial_test(VarInt(70));
    serial_test(VarInt(125));
    serial_test(VarInt(55));
}