use crate::{bits::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError}, types::{NetworkUserId, NetworkTypeId}};

/// Implements ManualBitSerialisation for tuple structs with a single `u32` field.
macro_rules! impl_single_u32 {
    ($type:ident) => {
        impl ManualBitSerialisation for $type {
            fn serialise(&self, writer: &mut impl BitWriter) {
                writer.write_u32(self.0);
            }
        
            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                Ok($type(reader.read_u32()?))
            }
        }       
    };
}

impl_single_u32!(NetworkUserId);
impl_single_u32!(NetworkTypeId);