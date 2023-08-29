use bevy::prelude::*;
use crate::prelude::*;
use crate::channels::outgoing::OutgoingOctetStringsAccessor;
use super::peer::UdpPeer;
use super::ports::PortBindings;

/// Sends octet strings using a sequential strategy.
pub(super) fn udp_send_packets_system_single(
    registry: Res<ChannelRegistry>,
    channels: Query<(Entity, &ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut clients: Query<(Entity, &UdpPeer), With<NetworkPeer>>,
    outgoing: OutgoingOctetStringsAccessor,
) {

}

/// Sends octet strings using a taskpool strategy.
pub(super) fn udp_send_packets_system_pooled(
    registry: Res<ChannelRegistry>,
    channels: Query<(Entity, &ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut clients: Query<(Entity, &UdpPeer), With<NetworkPeer>>,
    ports: Res<PortBindings>,
    outgoing: OutgoingOctetStringsAccessor,
) {

}