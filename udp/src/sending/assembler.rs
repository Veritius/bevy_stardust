//! Assembling octet strings into packets.

use bevy::prelude::*;
use bevy_stardust::prelude::*;

use crate::prelude::*;

fn assemble_packet<'a>(
    channels: &ChannelRegistry,
    peer_data: &mut UdpConnection,
    strings: impl Iterator<Item = (ChannelId, Entity, &'a OctetString)>,
) {

}