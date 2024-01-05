use std::{hash::Hasher, any::TypeId};

/// Pre-defined seed used in GxHasher.
pub(super) const STABLE_HASHER_SEED: i64 = 0x68066CFE6F752C27;

/// A stably hashable type, for comparing configurations across the network.
/// Since `#[derive(Hash)]` does not guarantee stability, this trait exists instead.
/// You should implement it manually.
/// 
/// This must always feed the same bytes into the hasher no matter the platform, Rust version, or compilation.
/// If this guarantee is not upheld, different compilations of the same application may become incompatible.
/// 
/// This guarantee becomes invalid if the choice of `H` is not a stable hashing algorithm.
/// Since Stardust always uses GxHasher with a set seed, this doesn't really matter much.
pub trait StableHash {
    /// Hashes the type through `H`.
    fn hash<H: Hasher>(&self, state: &mut H);
}

impl StableHash for () {
    fn hash<H: Hasher>(&self, state: &mut H) {}
}

macro_rules! impl_stablehash_simple {
    ($type:ident, $func:ident) => {
        impl StableHash for $type {
            fn hash<H: Hasher>(&self, state: &mut H) {
                state.$func(*self);
            }
        }
    };
}

impl_stablehash_simple!(u8, write_u8);
impl_stablehash_simple!(u16, write_u16);
impl_stablehash_simple!(u32, write_u32);
impl_stablehash_simple!(u64, write_u64);
impl_stablehash_simple!(u128, write_u128);
impl_stablehash_simple!(usize, write_usize);
impl_stablehash_simple!(i8, write_i8);
impl_stablehash_simple!(i16, write_i16);
impl_stablehash_simple!(i32, write_i32);
impl_stablehash_simple!(i64, write_i64);
impl_stablehash_simple!(i128, write_i128);
impl_stablehash_simple!(isize, write_isize);

impl StableHash for bool {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            true => state.write_u8(u8::MAX),
            false => state.write_u8(u8::MIN),
        }
    }
}

impl StableHash for &str {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes());
        state.write_u8(0xFF);
    }
}