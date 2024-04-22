use anyhow::{bail, Context, Error};
use bevy::prelude::*;
use bevy_stardust::prelude::*;
use unbytes::*;
use crate::prelude::*;
use crate::entities::ComponentSerialisationFunctions;
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

pub(super) fn receive_component_messages<T: Component>(
    mut commands: Commands,
    registry: Res<ChannelRegistry>,
    peers: Query<(Entity, &ReplicationPeer, &NetworkMessages<Incoming>, &NetworkEntityIds), With<NetworkPeer>>,
    mut comps: Query<&mut T, With<ReplicateEntity>>,
    serde_fns: Res<ComponentSerialisationFunctions<T>>,
) {
    let channel = registry.channel_id(std::any::TypeId::of::<ComponentReplicationChannel<T>>()).unwrap();
    for (peer, peer_meta, messages, ids) in peers.iter() {
        for message in messages.get(channel).iter().cloned() {
            let mut reader = Reader::new(message);

            // Try blocks when
            match (|| -> Result<(), Error> {
                let bt = reader.read_u8()
                    .context("Malformed network message")?;
                let hdr = ComponentMessageHeader::try_from(bt).map_err(|_| {
                    anyhow::anyhow!("Invalid message header value")
                })?;

                if peer_meta.side() != Side::Client { bail!("Received authority message from client") }
                let nid = NetworkEntityId::from(reader.read_array::<4>()
                    .context("Malformed network message")?);
                let eid = ids.get_ent_id(nid)
                    .context(format!("No entity ID associated with {nid:?}"))?;

                match hdr {
                    ComponentMessageHeader::Insert | ComponentMessageHeader::Update => {
                        let mut cmds = commands.get_entity(eid)
                            .context("Target entity for insertion did not exist")?;
                        let cmp = (serde_fns.0.deserialise)(reader.read_to_end())
                            .context("Couldn't deserialise component")?;

                        match hdr {
                            ComponentMessageHeader::Insert => {
                                cmds.try_insert(cmp);
                            },
                            ComponentMessageHeader::Update => {
                                let mut cm = comps.get_mut(eid)
                                    .context("Couldn't get replicated entity in query")?;

                                *cm = cmp;
                            },
                            _ => unreachable!(),
                        }
                    },

                    ComponentMessageHeader::Remove => {
                        commands.get_entity(eid)
                            .context("Target entity for removal did not exist")?
                            .remove::<T>();
                    },
                }

                Ok(())
            })() {
                Ok(_) => {},
                Err(err) => {
                    error!("Component replication error: {err}");
                },
            }
        }
    }
}