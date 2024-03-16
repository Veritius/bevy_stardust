use bevy_ecs::prelude::*;
use crate::Connection;
use super::Established;

pub(crate) fn established_breaking_system(
    mut connections: Query<(&mut Connection, &mut Established)>,
) {

}

pub(crate) fn established_packing_system(
    mut connections: Query<(&mut Connection, &mut Established)>,
) {

}