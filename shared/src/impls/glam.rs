use glam::{Vec2, Vec3, Vec3A, Vec4, Mat2, Mat3, Mat3A, Mat4, Quat, Affine2, Affine3A, DVec2, DVec3, DVec4, DMat2, DMat3, DMat4, DQuat, DAffine2, DAffine3, IVec2, IVec3, IVec4, UVec2, UVec3, UVec4, I64Vec4, I64Vec2, I64Vec3, U64Vec2, U64Vec3, U64Vec4, BVec2, BVec3, BVec4};
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
/// - `f32` or `f64`
/// - The type to implement on
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

macro_rules! impl_int_array {
    ($int:ident, $type:ident, $ints:expr) => {
        impl ManualBitSerialisation for $type {
            fn serialise(&self, writer: &mut impl BitWriter) {
                for i in self.to_array() {
                    let bytes = i.to_be_bytes();
                    let bytes_iter = bytes.iter().cloned();
                    writer.allocate_bytes(bytes.len());
                    writer.write_bytes(bytes_iter);
                }
            }
        
            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                let mut vals = [0; $ints];
                for z in 0..$ints {
                    vals[z] = $int::deserialise(reader)?;
                }
                Ok($type::from_array(vals))
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

impl_int_array!(i32, IVec2, 2);
impl_int_array!(i32, IVec3, 3);
impl_int_array!(i32, IVec4, 4);
impl_int_array!(u32, UVec2, 2);
impl_int_array!(u32, UVec3, 3);
impl_int_array!(u32, UVec4, 4);
impl_int_array!(i64, I64Vec2, 2);
impl_int_array!(i64, I64Vec3, 3);
impl_int_array!(i64, I64Vec4, 4);
impl_int_array!(u64, U64Vec2, 2);
impl_int_array!(u64, U64Vec3, 3);
impl_int_array!(u64, U64Vec4, 4);

impl ManualBitSerialisation for BVec2 {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.write_bit(self.x);
        writer.write_bit(self.y);
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> where Self: Sized {
        Ok(BVec2::new(
            reader.read_bit()?,
            reader.read_bit()?,
        ))
    }
}

impl ManualBitSerialisation for BVec3 {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.write_bit(self.x);
        writer.write_bit(self.y);
        writer.write_bit(self.z);
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> where Self: Sized {
        Ok(BVec3::new(
            reader.read_bit()?,
            reader.read_bit()?,
            reader.read_bit()?,
        ))
    }
}

impl ManualBitSerialisation for BVec4 {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.write_bit(self.x);
        writer.write_bit(self.y);
        writer.write_bit(self.z);
        writer.write_bit(self.w);
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> where Self: Sized {
        Ok(BVec4::new(
            reader.read_bit()?,
            reader.read_bit()?,
            reader.read_bit()?,
            reader.read_bit()?,
        ))
    }
}