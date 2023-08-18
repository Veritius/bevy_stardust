use crate::shared::{serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError}, integers::u24};

impl ManualBitSerialisation for u24 {
    fn serialise(&self, writer: &mut impl BitWriter) {
        writer.write_bytes(self.bytes().into_iter())
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        let bytes = TryInto::<[u8; 3]>::try_into(reader.read_bytes(3)?).unwrap();
        Ok(Self::from(bytes))
    }
}