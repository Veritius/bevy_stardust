pub const TRANSPORT_VERSION: &[u8] = b"bevy_stardust_udp/1";
pub const ACCEPTABLE_TRANSPORT_VERSIONS: &[&[u8]] = &[
    TRANSPORT_VERSION,
];

mod initial;
mod failure;