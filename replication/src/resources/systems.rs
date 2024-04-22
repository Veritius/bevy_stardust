use bevy::prelude::*;
use bevy_stardust::prelude::*;
use crate::prelude::*;
use super::messages::*;

pub(super) fn recv_resource_data_system<T: Resource>(
    mut res: ResMut<T>,
    ser: Res<ResourceSerialisationFunctions<T>>,
    registry: Res<ChannelRegistry>,
    peers: Query<(&ReplicationPeer, &NetworkMessages<Incoming>), With<NetworkPeer>>,
) {

}

pub(super) fn send_resource_data_system<T: Resource>(
    res: Res<T>,
    ser: Res<ResourceSerialisationFunctions<T>>,
    registry: Res<ChannelRegistry>,
    mut peers: Query<(&ReplicationPeer, &mut NetworkMessages<Incoming>), With<NetworkPeer>>,
) {

}