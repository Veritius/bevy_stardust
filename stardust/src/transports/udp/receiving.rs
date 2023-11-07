use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::prelude::*;
use crate::protocol::UniqueNetworkHash;
use crate::scheduling::NetworkScheduleData;
use super::connections::AllowNewConnections;
use super::ports::PortBindings;

/// Processes packets from bound ports using a task pool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    schedule: NetworkScheduleData,
    registry: Res<ChannelRegistry>,
    channels: Query<(Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut ports: ResMut<PortBindings>,
    hash: Res<UniqueNetworkHash>,
    allow_new: Res<AllowNewConnections>,
) {
    // Create task pool for parallel accesses
    let taskpool = TaskPoolBuilder::default()
        .thread_name("UDP pkt receive".to_string())
        .build();

    // Start reading bytes from all ports in parallel
    taskpool.scope(|s| {
        for (_, socket, peers) in ports.iter() {
            s.spawn(async move {

            });
        }
    });

    #[cfg(debug_assertions="true")]
    ports.confirm_reservation_emptiness();
}