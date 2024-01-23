use bevy::prelude::*;
use bevy_stardust::connections::peer::NetworkPeer;
use crate::{sockets::SocketManager, connection::UdpConnection};

pub(crate) fn packet_sender_system(
    mut connections: Query<(&mut NetworkPeer, &mut UdpConnection)>,
    mut sockets: ResMut<SocketManager>
) {

}