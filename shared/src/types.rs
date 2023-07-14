use crate::bits::{ManualBitSerialisation, BitstreamError};

/// A type that is replicated over the network.
pub struct NetworkTypeId(u32);
impl ManualBitSerialisation for NetworkTypeId {
    fn serialise(&self, writer: &mut impl crate::bits::BitWriter) {
        todo!()
    }

    fn deserialise(reader: &mut impl crate::bits::BitReader) -> Result<NetworkTypeId, BitstreamError> {
        todo!()
    }
}