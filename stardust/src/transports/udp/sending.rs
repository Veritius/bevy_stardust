use bevy::prelude::*;
use crate::prelude::*;
use super::peer::UdpPeer;

pub(super) fn udp_send_packets_system(
    registry: Res<ChannelRegistry>,
    channels: Query<(Entity, &ChannelData, Option<&ReliableChannel>, Option<&OrderedChannel>, Option<&FragmentedChannel>)>,
    mut clients: Query<(Entity, &UdpPeer), With<NetworkPeer>>,
) {

}