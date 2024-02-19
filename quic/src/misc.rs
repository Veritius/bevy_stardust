/// Minimum amount of bytes needed to represent a channel ID
pub(crate) fn bytes_for_channel_id(max: u32) -> u8 {
    match max {
        0..=255 => 1,
        256..=65535 => 2,
        65536..=16777216 => 3,
        _ => 4,
    }
}

#[test]
fn channel_id_bytes() {
    assert_eq!(bytes_for_channel_id(0), 1);
    assert_eq!(bytes_for_channel_id(u8::MAX as u32), 1);
    assert_eq!(bytes_for_channel_id(u8::MAX as u32 + 1), 2);
    assert_eq!(bytes_for_channel_id(u16::MAX as u32), 2);
    assert_eq!(bytes_for_channel_id(u16::MAX as u32 + 1), 3);
    assert_eq!(bytes_for_channel_id(2u32.pow(24)), 3);
    assert_eq!(bytes_for_channel_id(2u32.pow(24) + 1), 4);
    assert_eq!(bytes_for_channel_id(u32::MAX), 4);
}