use untrusted::{Reader, EndOfInput};

/// Tries to read a constant `N` bytes from a slice provided by a Reader, and returns a `[u8;N]` if successful.
// TODO: This could be better served by a dedicated integer-from-slice trait
pub(crate) fn slice_to_array<const N: usize>(reader: &mut Reader) -> Result<[u8; N], EndOfInput> {
    let slice = reader.read_bytes(N)?.as_slice_less_safe();
    let array = TryInto::<[u8; N]>::try_into(slice).map_err(|_| EndOfInput)?;
    Ok(array)
}

pub(crate) trait IntegerFromByteSlice {
    fn from_byte_slice(reader: &mut Reader) -> Result<Self, EndOfInput> where Self: Sized;
}

impl IntegerFromByteSlice for u16 {
    fn from_byte_slice(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(u16::from_be_bytes(slice_to_array::<2>(reader)?))
    }
}

impl IntegerFromByteSlice for u32 {
    fn from_byte_slice(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(u32::from_be_bytes(slice_to_array::<4>(reader)?))
    }
}

impl IntegerFromByteSlice for u64 {
    fn from_byte_slice(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(u64::from_be_bytes(slice_to_array::<8>(reader)?))
    }
}