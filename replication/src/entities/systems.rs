use bevy::prelude::*;
use bevy_stardust::prelude::*;
use unbytes::*;
use crate::prelude::*;
use super::{ids::*, messages::*};

// When Bevy 0.14 releases, use hooks (pr #10756)
pub(super) fn ensure_id_component(
    world: &mut World,
) {
    let mut q = world.query_filtered::<(Entity, &ReplicationPeer), Without<NetworkEntityIds>>();
    let col = q.iter(&world).map(|(id,com)| (id, com.side())).collect::<Vec<_>>();
    for (ent, side) in col {
        world.entity_mut(ent).insert(NetworkEntityIds::new(side));
    }
}

pub(super) fn receive_entity_messages(
    mut commands: Commands,
    registry: Res<ChannelRegistry>,
    mut peers: Query<(Entity, &ReplicationPeer, &NetworkMessages<Incoming>, &mut NetworkEntityIds), With<NetworkPeer>>,
) {
    let channel = registry.channel_id(std::any::TypeId::of::<EntityReplicationChannel>()).unwrap();
    for (peer, peer_meta, messages, mut ids) in peers.iter_mut() {
        for message in messages.get(channel).iter().cloned() {
            let mut reader = Reader::new(message);

            // Try blocks when
            let _: Result<(), ()> = (|| {
                let bt = reader.read_u8().map_err(|_| ())?;
                let msg = EntityMessageHeader::try_from(bt)?;

                match msg {
                    EntityMessageHeader::Spawn => {
                        if peer_meta.side() != Side::Client { return Err(()); }
                        let nid = NetworkEntityId::from(reader.read_array::<4>().map_err(|_| ())?);
                        let eid = commands.spawn(ReplicateEntity {
                            ids: {
                                let mut ids = AssociatedNetworkIds::default();
                                ids.insert(peer, nid);
                                ids
                            }
                        }).id();
                        ids.add_pair(eid, nid);
                    },

                    EntityMessageHeader::Despawn => {
                        if peer_meta.side() != Side::Client { return Err(()); }
                        let nid = NetworkEntityId::from(reader.read_array::<4>().map_err(|_| ())?);
                        let eid = ids.remove_net_id(nid).ok_or(())?;
                        commands.entity(eid).despawn();
                    },
                }

                Ok(())
            })();
        }
    }
}