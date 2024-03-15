use crate::appdata::NetworkVersionData;
use crate::connection::reliability::ReliablePacketHeader;

#[derive(Debug)]
pub(super) struct ClientHelloPacket {
    pub transport: NetworkVersionData,
    pub application: NetworkVersionData,
    pub sequence_identifier: u16,
}

#[derive(Debug)]
pub(super) struct ServerHelloPacket {
    pub transport: NetworkVersionData,
    pub application: NetworkVersionData,
    pub reliability: ReliablePacketHeader,
}

#[derive(Debug)]
pub(super) struct ClientFinalisePacket {
    pub reliability: ReliablePacketHeader,
}