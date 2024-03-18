use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::{plugin::PluginConfiguration, Connection};
use super::Established;

macro_rules! try_unwrap {
    ($id:tt, $st:expr) => {
        match $st {
            Ok(val) => val,
            Err(_) => { continue $id; }
        }
    };
}

pub(crate) fn established_packet_reader_system(
    mut connections: Query<(&mut Connection, &mut Established, &mut NetworkMessages<Incoming>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state, mut incoming)| {
        todo!()
    });
}

pub(crate) fn established_packet_builder_system(
    registry: ChannelRegistry,
    config: Res<PluginConfiguration>,
    mut connections: Query<(&mut Connection, &mut Established, &NetworkMessages<Outgoing>)>,
) {
    // Process all connections in parallel
    connections.par_iter_mut().for_each(|(mut meta, mut state, outgoing)| {
        todo!();
    });
}