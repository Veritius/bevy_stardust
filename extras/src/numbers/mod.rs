//! Types for working with numbers, such as efficient encoding and easier logic.

mod sequence;
mod varint;

pub use sequence::{Sequence, Sequential};
pub use varint::VarInt;