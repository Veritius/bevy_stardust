use std::{rc::Rc, sync::Arc};
use crate::shared::serialisation::{ManualBitSerialisation, BitWriter, BitReader, BitstreamError};

macro_rules! impl_pointer {
    ($t:ident, $get:ident) => {
        impl<T: ManualBitSerialisation> ManualBitSerialisation for $t<T> {
            fn serialise(&self, writer: &mut impl BitWriter) {
                self.$get().serialise(writer);
            }
        
            fn deserialise(reader: &mut impl BitReader) -> Result<Self, BitstreamError> {
                Ok(Self::new(T::deserialise(reader)?))
            }
        }
    };
}

impl_pointer!(Box, as_ref);
impl_pointer!(Rc, as_ref);
impl_pointer!(Arc, as_ref);