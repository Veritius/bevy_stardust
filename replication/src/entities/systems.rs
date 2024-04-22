use anyhow::{bail, Context, Error};
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
            match (|| -> Result<(), Error> {
                let bt = reader.read_u8()
                    .context("Malformed network message")?;
                let hdr = EntityMessageHeader::try_from(bt).map_err(|_| {
                    anyhow::anyhow!("Invalid message header value")
                })?;

                if peer_meta.side() != Side::Client { bail!("Received authority message from client") }
                let nid = NetworkEntityId::from(reader.read_array::<4>()
                    .context("Malformed network message")?);

                match hdr {
                    EntityMessageHeader::Spawn => {
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
                        let eid = ids.remove_net_id(nid)
                            .context(format!("No entity ID associated with {nid:?}"))?;
                        commands.entity(eid).despawn();
                    },
                }

                Ok(())
            })() {
                Ok(_) => {},
                Err(err) => {
                    error!("Entity replication error: {err}");
                },
            }
        }
    }
}

pub(super) fn send_entity_messages(
    mut commands: Commands,
    registry: Res<ChannelRegistry>,
    mut peers: Query<(Entity, &ReplicationPeer, &mut NetworkMessages<Outgoing>, &mut NetworkEntityIds), With<NetworkPeer>>,
    changes: Query<(Entity, Ref<ReplicateEntity>)>,
    removals: RemovedComponents<ReplicateEntity>,
) {
    let channel = registry.channel_id(std::any::TypeId::of::<EntityReplicationChannel>()).unwrap();
    let changed = changes.iter().filter(|(_, c)| c.is_changed()).map(|(e, r)| (e, r.is_added())).collect::<Vec<_>>();
    peers.par_iter_mut().for_each(|(peer, peer_meta, mut messages, mut ids)| {
        for (ent, added) in changed.iter().cloned() {
            todo!()
        }
    });
}