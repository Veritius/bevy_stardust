#[derive(Debug)]
#[repr(u8)]
pub(super) enum PacketFrame {
    Padding = 0,
    Acknowledgement = 1,
    Payload = 2,
}