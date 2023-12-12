/// Returns how many bytes are needed to store all possible registered channel IDs.
#[inline]
pub(crate) fn bytes_for_channel_ids(val: u32) -> u8 {
    match val {
        0..=254 => 1,
        255..=65534 => 2,
        65535..=16777215 => 3,
        _ => 4,
    }
}