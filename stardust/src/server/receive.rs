use std::{collections::BTreeMap, sync::Mutex};
use bevy::{prelude::*, tasks::TaskPool, ecs::system::{SystemState, SystemParam}};
use crate::shared::{protocol::{MAX_PACKET_LENGTH, Protocol}, channel::{ChannelId, Channel}};
use super::clients::Client;

/// Used in channel message processing systems to get incoming messages from a channel.
#[derive(SystemParam)]
pub struct ChannelMessageSets<'w> {
    protocol: Res<'w, Protocol>,
    map: Res<'w, ChannelMap>,
}

impl ChannelMessageSets<'_> {
    /// Removes the channel data from the map. This will cause other systems requesting the same channel data afterwards to panic.
    pub fn take<T: Channel>(&self) -> Vec<(Entity, Box<[u8]>)> {
        let channel = self.protocol.get_id::<T>().expect("Channel didn't exist");
        self.map.0.lock().unwrap().remove(&channel).expect("Requested channel was not in map (already removed by another system?)")
    }
}

#[derive(Resource)]
struct ChannelMap(Mutex<BTreeMap<ChannelId, Vec<(Entity, Box<[u8]>)>>>);

impl FromWorld for ChannelMap {
    fn from_world(world: &mut World) -> Self {
        let protocol = world.resource::<Protocol>();

        // Preallocate space
        let mut map: BTreeMap<ChannelId, Vec<(Entity, Box<[u8]>)>> = BTreeMap::new();
        for i in 0..protocol.channels() {
            let id = ChannelId::new(i);
            let cfg = protocol.channel_config(id).expect(&format!("Channel {:?} had no associated config", id));
            map.insert(id, Vec::with_capacity(cfg.messages_per_tick_server));
        }

        Self(Mutex::new(map))
    }
}

/// Minimum packet length in bytes. Packets with less information than this will be discarded.
const MIN_PACKET_BYTES: usize = 7;

// Receives raw packet information from all UDP sockets associated with clients.
pub(super) fn receive_packets_system(
    world: &mut World,
) {
    let mut state: SystemState<(
        Res<Protocol>,
        Query<(Entity, &Client)>,
        ResMut<ChannelMap>,
    )> = SystemState::new(world);
    let state = state.get_mut(world);
    
    let protocol = state.0.as_ref();
    let clients = state.1;
    let mut channel_map = state.2.0.lock().unwrap();

    let pool = TaskPool::new();

    // Read packets from all clients
    let mut packet_groups = pool.scope(|s| {
        for (client_id, client_comp) in clients.iter() {
            let client_id = client_id.clone();
            s.spawn(async move {
                let mut packets = vec![];
                let mut buffer = [0u8; MAX_PACKET_LENGTH];
                loop {
                    if let Ok(bytes) = client_comp.socket.recv(&mut buffer) {
                        // Some early checks
                        // Source address is already filtered by UdpSocket::connect()
                        if bytes < MIN_PACKET_BYTES { continue; } // Not enough data to be a valid packet, discard
                        let protocol_id = u32::from_be_bytes(buffer[0..=3].try_into().unwrap());
                        if protocol_id != protocol.id() { continue; } // Wrong protocol ID, discard packet

                        // Get channel ID
                        let channel_id = ChannelId::from_bytes(buffer[4..=7].try_into().unwrap());

                        // Copy relevant buffer data into vec
                        let midx = bytes - MIN_PACKET_BYTES - 1;
                        let mut packet = Vec::with_capacity(midx);
                        for i in (MIN_PACKET_BYTES + 1)..(midx) {
                            packet.push(buffer[i]);
                        }

                        // Add message information to list
                        packets.push((client_id, channel_id, packet.into_boxed_slice()));
                    } else {
                        // No more data to read
                        break;
                    }
                }

                packets
            });
        }
    });

    // Sort packets into channels
    for _ in 0..=packet_groups.len() {
        let mut packet_group = packet_groups.pop().unwrap();
        for _ in 0..=packet_group.len() {
            let (sender, channel, packet) = packet_group.pop().unwrap();
            channel_map.get_mut(&channel).unwrap().push((sender, packet));
        }
    }
}