use bevy::prelude::*;
use crate::bits::ManualBitSerialisation;
use super::{config::ReplicatedComponentData, markers::ReplicatedEntity};

pub(crate) fn replication_send_system_bitstream<T: Component + ManualBitSerialisation>(
    config: Res<ReplicatedComponentData<T>>,
    changed: Query<(&ReplicatedEntity, &T), Changed<T>>,
) {
    todo!();
    for comp in changed.iter() {

    }
}

pub(crate) fn replication_send_system_reflected<T: Component + Reflect>(
    config: Res<ReplicatedComponentData<T>>,
    changed: Query<(&ReplicatedEntity, &T), Changed<T>>,
) {
    todo!();
    for comp in changed.iter() {

    }
}