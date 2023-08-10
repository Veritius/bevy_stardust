use crate::shared::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

/// Implements serialisation for integers and other types where endianness matters.
/// Always converts to big endian for network transport.
/// Takes the type and the number of bytes as arguments.
macro_rules! impl_manual_be {
    ($t:ident, $n:expr) => {
        impl ManualBitSerialisation for $t {
            fn serialise(&self, writer: &mut impl BitWriter) {
                writer.write_bytes(self.to_be_bytes().iter().cloned())
            }

            fn deserialise(reader: &mut impl BitReader) -> Result<$t, BitstreamError> {
                match reader.read_bytes($n) {
                    Ok(bytes) => {
                        let byte_array: [u8; $n] = bytes.as_slice().try_into().expect("Byte vec should have been perfectly sized");
                        Ok($t::from_be_bytes(byte_array))
                    },
                    Err(error) => Err(error),
                }
            }
        }
    };
}

impl_manual_be!(u8, 1);
impl_manual_be!(u16, 2);
impl_manual_be!(u32, 4);
impl_manual_be!(u64, 8);
impl_manual_be!(u128, 16);
impl_manual_be!(i8, 1);
impl_manual_be!(i16, 2);
impl_manual_be!(i32, 4);
impl_manual_be!(i64, 8);
impl_manual_be!(i128, 16);
impl_manual_be!(f32, 4);
impl_manual_be!(f64, 8);

impl ManualBitSerialisation for bool {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.write_bit(*self)
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        reader.read_bit()
    }
}

impl ManualBitSerialisation for usize {
    fn serialise(&self, writer: &mut impl BitWriter) {
        let n = *self as u64;
        n.serialise(writer);
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> where Self: Sized {
        let n = u64::deserialise(reader)?;
        Ok(n as usize)
    }
}

impl ManualBitSerialisation for String {
    fn serialise(&self, writer: &mut impl BitWriter) {
        let bytes = self.bytes();
        let length = bytes.len() as u32;
        length.serialise(writer);
        writer.write_bytes(bytes);
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        let length = u32::deserialise(reader)?;
        let bytes = reader.read_bytes(length as usize)?;
        if let Ok(string) = String::from_utf8(bytes) {
            return Ok(string)
        } else {
            return Err(BitstreamError)
        }
    }
}