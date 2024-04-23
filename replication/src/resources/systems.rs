use bevy::{ecs::system::SystemChangeTick, prelude::*};
use bevy_stardust::prelude::*;
use crate::{change::NetChangeTracking, prelude::*};
use super::{change::NetRes, messages::*};

pub(super) fn recv_resource_data_system<T: ReplicableResource>(
    mut res: ResMut<T>,
    mut chg: ResMut<NetChangeTracking<T>>,
    ticks: SystemChangeTick,
    ser: Res<ResourceSerialisationFunctions<T>>,
    registry: Res<ChannelRegistry>,
    peers: Query<(&ReplicationPeer, &NetworkMessages<Incoming>), With<NetworkPeer>>,
) {
    let channel = registry.channel_id(std::any::TypeId::of::<ResourceReplicationMessages<T>>()).unwrap();
    for (peer, messages) in peers.iter() {
        if peer.side() == Side::Client { todo!(); }
        for message in messages.get(channel).iter().cloned() {
            let t = match (ser.fns.deserialise)(message) {
                Ok(t) => t,
                Err(err) => {
                    error!("Error while deserialising replicated resource {}: {err}",
                        std::any::type_name::<T>());

                    continue;
                },
            };

            *res = t;
            chg.set_change_tick(ticks.this_run());
        }
    }
}

pub(super) fn send_resource_data_system<T: ReplicableResource>(
    res: NetRes<T>,
    ser: Res<ResourceSerialisationFunctions<T>>,
    registry: Res<ChannelRegistry>,
    mut peers: Query<(&ReplicationPeer, &mut NetworkMessages<Outgoing>), With<NetworkPeer>>,
) {
    // Only replicate if the application made the change
    if !res.is_changed_by_application() { return; }

    // Serialise ahead of time
    let bytes = match (ser.fns.serialise)(&res) {
        Ok(v) => v,
        Err(err) => {
            error!("Error while serialising replicated resource {}: {err}",
                std::any::type_name::<T>());

            return;
        },
    };

    let channel = registry.channel_id(std::any::TypeId::of::<ResourceReplicationMessages<T>>()).unwrap();
    for (_peer, mut messages) in peers.iter_mut() {
        messages.push(channel, bytes.clone());
    }
}