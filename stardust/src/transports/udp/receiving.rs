use std::io::ErrorKind;
use bevy::ecs::system::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::TaskPoolBuilder;
use crate::messages::incoming::NetworkMessageStorage;
use crate::prelude::*;
use crate::protocol::UniqueNetworkHash;
use crate::scheduling::NetworkScheduleData;
use super::connections::{AllowNewConnections, UdpConnection};
use super::parallel::DeferredCommandQueue;
use super::ports::PortBindings;

/// Minimum amount of octets in a packet before it's ignored.
const MIN_OCTETS: usize = 3;

/// Processes packets from bound ports using a task pool strategy.
pub(super) fn receive_packets_system(
    mut commands: Commands,
    mut peers: Query<(Entity, &mut UdpConnection, Option<&mut NetworkMessageStorage>)>,
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
    let mut task_commands = taskpool.scope(|s| {
        for (_, socket, peers) in ports.iter() {
            s.spawn(async move {
                let mut commands = CommandQueue::default();
                let mut buffer = [0u8; 1472];

                loop {
                    // Try to read a packet from the socket
                    let (octets_read, origin) = match socket.recv_from(&mut buffer) {
                        Ok(s) => s,
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            // We've run out of packets to read
                            break
                        },
                        Err(e) => {
                            // Actual I/O error, stop reading
                            error!("Error while reading packets: {e}");
                            return commands
                        },
                    };

                    // Check length of message
                    if octets_read < MIN_OCTETS { continue }
                }

                // Return deferred mutations that this task wants to perform
                return commands
            });
        }
    });

    // Add commands from tasks to Commands
    for command in task_commands.drain(..) {
        commands.add(DeferredCommandQueue(command))
    }

    #[cfg(debug_assertions="true")]
    ports.confirm_reservation_emptiness();
}