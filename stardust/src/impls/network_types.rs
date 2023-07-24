use crate::{shared::{serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError}, user::NetworkUserId}, replication::entities::ReplicatedEntityId};

/// Implements ManualBitSerialisation for tuple structs with a single `u32` field.
macro_rules! impl_single {
    ($type:ident, u32) => {
        impl ManualBitSerialisation for $type {
            fn serialise(&self, writer: &mut impl BitWriter) {
                writer.write_u32(self.0);
            }
        
            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                Ok($type(reader.read_u32()?))
            }
        }       
    };
    ($type:ident, u64) => {
        impl ManualBitSerialisation for $type {
            fn serialise(&self, writer: &mut impl BitWriter) {
                writer.write_u64(self.0);
            }
        
            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                Ok($type(reader.read_u64()?))
            }
        }       
    };
}

impl_single!(NetworkUserId, u32);
impl_single!(ReplicatedEntityId, u64);