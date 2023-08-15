use bevy::prelude::*;
use crate::{shared::channels::outgoing::OutgoingOctetStringsAccessor, server::{clients::Client, prelude::*}};
use super::UdpClient;

pub(super) fn send_packets_system(
    // registry: Res<ChannelRegistry>,
    // channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    outgoing: OutgoingOctetStringsAccessor,
    clients: Query<&UdpClient, With<Client>>,
) {
    
}