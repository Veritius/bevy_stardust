use bevy::prelude::*;
use crate::{components::ReplicatedEntity, bits::ManualBitSerialisation};
use super::config::ReplicatedComponentData;

pub(crate) fn reflection_send_system_bitstream<T: Component + ManualBitSerialisation>(
    config: Res<ReplicatedComponentData<T>>,
    changed: Query<(&ReplicatedEntity, &T), Changed<T>>,
) {
    for comp in changed.iter() {

    }
}

pub(crate) fn replication_send_system_reflected<T: Component + Reflect>(
    config: Res<ReplicatedComponentData<T>>,
    changed: Query<(&ReplicatedEntity, &T), Changed<T>>,
) {
    for comp in changed.iter() {

    }
}