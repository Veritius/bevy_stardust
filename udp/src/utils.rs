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

#[inline]
pub(crate) const fn sequence_greater_than(s1: u16, s2: u16) -> bool {
    ((s1>s2)&&(s1-s2<=32768))||((s1<s2)&&(s2-s1>32768))
}