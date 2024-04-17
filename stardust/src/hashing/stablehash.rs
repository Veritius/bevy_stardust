use std::hash::Hasher;

/// Pre-defined seed used in GxHasher.
pub const STABLE_HASHER_SEED: i64 = 0x68066CFE6F752C27;

/// A stably hashable type, for comparing data across machines.
/// 
/// Implementors must uphold the following guarantees:
/// - The hash is fully deterministic and only changes with crate versions.
///     - Derive macros should not be used without `#[repr(C)]`, as the Rust compiler can reorder fields.
///     - Semantic versioning applies: a change in a hashed value should be considered a breaking change.
/// - The hash must be the same regardless of CPU architecture, platform, Rust version, or compilation.
/// - Endianness must be taken into account when using the `Hasher`.
///     - `write` and `write_u8` are fine and don't change with endianness.
///     - Other functions like `write_u32` should have consistent endianness.
///     - Use `to_be` or `to_le` on integer types.
///
/// While `StableHash` is generic over the `Hasher`,
/// you should only use a hasher that is known to be stable.
/// Currently, this is `gxhash`, but this may change in future.
pub trait StableHash {
    /// Hashes the type through `H`.
    fn hash<H: Hasher>(&self, state: &mut H);
}

impl StableHash for () {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

macro_rules! impl_stablehash_simple {
    ($type:ident) => {
        impl StableHash for $type {
            fn hash<H: Hasher>(&self, state: &mut H) {
                state.write(&self.to_be_bytes());
            }
        }
    };
}

impl_stablehash_simple!(u8);
impl_stablehash_simple!(u16);
impl_stablehash_simple!(u32);
impl_stablehash_simple!(u64);
impl_stablehash_simple!(u128);
impl_stablehash_simple!(usize);
impl_stablehash_simple!(i8);
impl_stablehash_simple!(i16);
impl_stablehash_simple!(i32);
impl_stablehash_simple!(i64);
impl_stablehash_simple!(i128);
impl_stablehash_simple!(isize);

impl StableHash for &[u8] {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(self);
    }
}

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

// Simple (and probably ineffective) check for hash stability in gxhash and our StableHash impls.
// The reference point is a Surface Pro 7 with an Intel i5-1035G4 (8) @ 3.700GHz CPU.
// The computer is running Linux Mint 21.2 x86_64 on kernel 6.6.6-surface-1.
// Compiled on rustc 1.72.1 (d5c2e9c34 2023-09-13) on the release channel.
// gxhash is version 3.0.0 exactly
// If this test ever fails, you should report it immediately.
#[test]
fn hash_stability_check() {
    let mut hasher = gxhash::GxHasher::with_seed(STABLE_HASHER_SEED);

    false.hash(&mut hasher);
    123456789u32.hash(&mut hasher);
    123456789i32.hash(&mut hasher);
    "Hello, World!".hash(&mut hasher);

    let hash = hasher.finish();
    assert_eq!(hash, 0xCE46E06873D99619);
}