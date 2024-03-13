use untrusted::{Reader, EndOfInput};

/// Tries to read a constant `N` bytes from a slice provided by a Reader, and returns a `[u8;N]` if successful.
// TODO: This could be better served by a dedicated integer-from-slice trait
pub(crate) fn slice_to_array<const N: usize>(reader: &mut Reader) -> Result<[u8; N], EndOfInput> {
    let slice = reader.read_bytes(N)?.as_slice_less_safe();
    let array = TryInto::<[u8; N]>::try_into(slice).map_err(|_| EndOfInput)?;
    Ok(array)
}