// Variable length integer implementation based on RFC 9000 (QUIC)
// Some code taken from quinn-proto, licensed under the MIT License
// https://github.com/quinn-rs/quinn/blob/a2a214b968fbcbc9aa66aba4393851b3d6ab5b49/quinn-proto/src/varint.rs
// Full copy of the license is here: https://github.com/quinn-rs/quinn/blob/a2a214b968fbcbc9aa66aba4393851b3d6ab5b49/LICENSE-MIT

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

impl VarInt {
    pub const MAX: Self = Self((1 << 62) - 1);
    pub const MIN: Self = Self(0);

    fn size(self) -> usize {
        let x = self.0;
        if x < 2u64.pow(6) {
            1
        } else if x < 2u64.pow(14) {
            2
        } else if x < 2u64.pow(30) {
            3
        } else if x < 2u64.pow(62) {
            8
        } else {
            unreachable!("bad varint");
        }
    }

    fn read(reader: &mut Reader) -> Result<Self, EndOfInput> {
        let fb = reader.read_array::<1>()?[0];
        todo!()
    }

    fn write<B: BufMut>(&self, buf: &mut B) {
        let x = self.0;
        if x < 2u64.pow(6) {
            buf.put_u8(x as u8);
        } else if x < 2u64.pow(14) {
            buf.put_u16(0b01 << 14 | x as u16);
        } else if x < 2u64.pow(30) {
            buf.put_u32(0b10 << 30 | x as u32);
        } else if x < 2u64.pow(62) {
            buf.put_u64(0b11 | x);
        } else {
            unreachable!("bad varint");
        }
    }
}

#[test]
fn debug_test() {
    println!("{:b}", u32::MAX);
}