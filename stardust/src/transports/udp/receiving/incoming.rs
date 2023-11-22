use std::net::UdpSocket;
use crate::transports::udp::{connections::{UdpConnection, Disconnected}, ports::PortBindings, COMPAT_GOOD_VERSIONS, TRANSPORT_IDENTIFIER};

pub(super) fn process_pending_incoming(
    message: &[u8],
    socket: &UdpSocket,
    connection: &mut UdpConnection,
    protocol: u64,
    ports: &PortBindings,
) {
    // Basic requirements for an initial packet
    if message.len() < 22 {
        connection.status = Disconnected::HandshakeMalformedPacket.into();
        return;
    }

    let identifier = u64::from_be_bytes(message[0..8].try_into().unwrap());
    if identifier != TRANSPORT_IDENTIFIER {
        connection.status = Disconnected::HandshakeUnknownTransport { identifier }.into();
        return;
    }

    let version = u32::from_be_bytes(message[8..12].try_into().unwrap());
    if !COMPAT_GOOD_VERSIONS.contains(&version) {
        connection.status = Disconnected::HandshakeWrongVersion { version }.into();
        return;
    }

    let other_protocol = u64::from_be_bytes(message[12..20].try_into().unwrap());
    if other_protocol != protocol {
        connection.status = Disconnected::HandshakeWrongProtocol { protocol }.into();
        return;
    }

    // Use their sequence id in Reliability's remote field
    connection.reliability.remote = u16::from_be_bytes(message[20..22].try_into().unwrap());
}