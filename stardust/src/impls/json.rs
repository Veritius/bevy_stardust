use json::{JsonValue, number::Number};
use crate::shared::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

impl ManualBitSerialisation for JsonValue {
    fn serialise(&self, writer: &mut impl BitWriter) {
        let dumped = self.dump();
        dumped.serialise(writer);
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        let json = String::deserialise(reader)?;
        let json = json::parse(&json);
        if json.is_err() { return Err(BitstreamError) }
        Ok(json.unwrap())
    }
}

impl ManualBitSerialisation for Number {
    fn serialise(&self, writer: &mut impl BitWriter) {
        let (positive, exponent, mantissa) = self.as_parts();
        positive.serialise(writer);
        exponent.serialise(writer);
        mantissa.serialise(writer);
    }

    fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
        let positive = bool::deserialise(reader)?;
        let exponent = u64::deserialise(reader)?;
        let mantissa = i16::deserialise(reader)?;
        Ok(Number::from_parts(positive, exponent, mantissa))
    }
}