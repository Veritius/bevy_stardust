use std::sync::Mutex;
use bevy::{prelude::*, tasks::TaskPool};
use once_cell::sync::Lazy;
use crate::{shared::channels::outgoing::OutgoingOctetStringsAccessor, server::clients::Client};
use super::{UdpClient, ports::PortBindings};

pub(super) fn send_packets_system(
    // registry: Res<ChannelRegistry>,
    // channels: Query<(&ChannelData, Option<&OrderedChannel>, Option<&ReliableChannel>, Option<&FragmentedChannel>)>,
    mut ports: ResMut<PortBindings>,
    clients: Query<&UdpClient, With<Client>>,
    outgoing: OutgoingOctetStringsAccessor,
) {
    // Create task pool
    let pool = TaskPool::new();

    // List of clients to remove in case of a mistake
    let port_removals = Lazy::new(|| Mutex::new(Vec::with_capacity(1)));

    // Intentional borrow to prevent moves
    let clients = &clients;
    let outgoing = &outgoing;
    let port_removals = &port_removals;

    // Create tasks for all ports
    pool.scope(|s| {
        for port in ports.iter() {
            s.spawn(async move {
                let (port, socket, cls) = port;
                for cl in cls {
                    // Get client entity
                    let Ok(ret) = clients.get(*cl) else {
                        error!("Entity {:?} was in PortBindings but was not a client", *cl);
                        port_removals.lock().unwrap().push(*cl);
                        continue;
                    };

                    // Iterate _all_ outgoing octet strings
                    let iter = outgoing.by_client(*cl);
                }
            });
        }
    });

    // Remove clients if there's an issue
    let lock = port_removals.lock().unwrap();
    for v in lock.iter() {
        ports.remove_client(*v);
    }
}