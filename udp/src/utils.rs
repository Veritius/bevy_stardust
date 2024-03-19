use untrusted::{Reader, EndOfInput};

/// Tries to read a constant `N` bytes from a slice provided by a Reader, and returns a `[u8;N]` if successful.
pub(crate) fn slice_to_array<const N: usize>(reader: &mut Reader) -> Result<[u8; N], EndOfInput> {
    let mut array = [0u8; N];
    let slice = reader.read_bytes(N)?.as_slice_less_safe();
    array.copy_from_slice(slice);
    Ok(array)
}

pub(crate) trait FromByteReader {
    fn from_byte_slice(reader: &mut Reader) -> Result<Self, EndOfInput> where Self: Sized;
}

impl FromByteReader for u16 {
    fn from_byte_slice(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(u16::from_be_bytes(slice_to_array::<2>(reader)?))
    }
}

impl FromByteReader for u32 {
    fn from_byte_slice(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(u32::from_be_bytes(slice_to_array::<4>(reader)?))
    }
}

impl FromByteReader for u64 {
    fn from_byte_slice(reader: &mut Reader) -> Result<Self, EndOfInput> {
        Ok(u64::from_be_bytes(slice_to_array::<8>(reader)?))
    }
}