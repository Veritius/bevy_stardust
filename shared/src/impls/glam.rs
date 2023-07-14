use glam::{Vec2, Vec3, Vec3A, Vec4, Mat2, Mat3, Mat3A, Mat4, Quat, Affine2, Affine3A, DVec2, DVec3, DVec4, DMat2, DMat3, DMat4, DQuat, DAffine2, DAffine3};
use crate::bits::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

macro_rules! write_f32 {
    ($writer:ident, $($value:expr),+) => {{
        $(
            $writer.write_u32($value.to_bits());
        )*
    }}
}

macro_rules! read_f32 {
    ($reader:ident) => {
        f32::from_bits($reader.read_u32()?)
    }
}

macro_rules! write_f64 {
    ($writer:ident, $($value:expr),+) => {{
        $(
            $writer.write_u64($value.to_bits());
        )*
    }}
}

macro_rules! read_f64 {
    ($reader:ident) => {
        f64::from_bits($reader.read_u64()?)
    }
}

/// Implements `ManualBitSerialisation` for types that can be turned to and from arrays of floating point numbers.
/// Usage example: `impl_float_array!(Vec2, 2, to_array, from_array, owned);`
/// 
/// The arguments are as follows:
/// - The type to implement on
/// - `f32` or `f64`
/// - The amount of floats in the array
/// - The to-method, the from-method
/// - `owned` (disables the borrow given to `to-method` as an argument)
macro_rules! impl_float_array {
    (f32, $type:ident, $i:expr, $t:ident, $f:ident) => {
        impl ManualBitSerialisation for $type {
            fn serialise(&self, writer: &mut impl BitWriter) {
                for i in self.$t().iter() {
                    let i = *i;
                    write_f32!(writer, i);
                }
            }

            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                let mut c = [f32::NAN; $i];
                for i in 0..4 {
                    c[i] = read_f32!(reader);
                }
                Ok($type::$f(&c))
            }
        }
    };

    (f32, $type:ident, $i:expr, $t:ident, $f:ident, owned) => {
        impl ManualBitSerialisation for $type {
            fn serialise(&self, writer: &mut impl BitWriter) {
                for i in self.$t().iter() {
                    let i = *i;
                    write_f32!(writer, i);
                }
            }

            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                let mut c = [f32::NAN; $i];
                for i in 0..4 {
                    c[i] = read_f32!(reader);
                }
                Ok($type::$f(c))
            }
        }
    };

    (f64, $type:ident, $i:expr, $t:ident, $f:ident) => {
        impl ManualBitSerialisation for $type {
            fn serialise(&self, writer: &mut impl BitWriter) {
                for i in self.$t().iter() {
                    let i = *i;
                    write_f64!(writer, i);
                }
            }

            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                let mut c = [f64::NAN; $i];
                for i in 0..4 {
                    c[i] = read_f64!(reader);
                }
                Ok($type::$f(&c))
            }
        }
    };

    (f64, $type:ident, $i:expr, $t:ident, $f:ident, owned) => {
        impl ManualBitSerialisation for $type {
            fn serialise(&self, writer: &mut impl BitWriter) {
                for i in self.$t().iter() {
                    let i = *i;
                    write_f64!(writer, i);
                }
            }

            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                let mut c = [f64::NAN; $i];
                for i in 0..4 {
                    c[i] = read_f64!(reader);
                }
                Ok($type::$f(c))
            }
        }
    };
}

impl_float_array!(f32, Vec2, 2, to_array, from_array, owned);
impl_float_array!(f32, Vec3, 3, to_array, from_array, owned);
impl_float_array!(f32, Vec3A, 3, to_array, from_array, owned);
impl_float_array!(f32, Vec4, 4, to_array, from_array, owned);
impl_float_array!(f32, Mat2, 4, to_cols_array, from_cols_array);
impl_float_array!(f32, Mat3, 9, to_cols_array, from_cols_array);
impl_float_array!(f32, Mat3A, 9, to_cols_array, from_cols_array);
impl_float_array!(f32, Mat4, 16, to_cols_array, from_cols_array);
impl_float_array!(f32, Quat, 4, to_array, from_array, owned);
impl_float_array!(f32, Affine2, 6, to_cols_array, from_cols_array);
impl_float_array!(f32, Affine3A, 12, to_cols_array, from_cols_array);
impl_float_array!(f64, DVec2, 2, to_array, from_array, owned);
impl_float_array!(f64, DVec3, 3, to_array, from_array, owned);
impl_float_array!(f64, DVec4, 4, to_array, from_array, owned);
impl_float_array!(f64, DMat2, 4, to_cols_array, from_cols_array);
impl_float_array!(f64, DMat3, 9, to_cols_array, from_cols_array);
impl_float_array!(f64, DMat4, 16, to_cols_array, from_cols_array);
impl_float_array!(f64, DQuat, 4, to_array, from_array, owned);
impl_float_array!(f64, DAffine2, 6, to_cols_array, from_cols_array);
impl_float_array!(f64, DAffine3, 12, to_cols_array, from_cols_array);