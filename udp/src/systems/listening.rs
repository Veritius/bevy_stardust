use std::{collections::BTreeMap, sync::Mutex, io::ErrorKind};
use bevy::prelude::*;
use bevy_stardust::connections::peer::NetworkPeer;
use crate::{sockets::SocketManager, connection::UdpConnection};

pub(crate) fn packet_listener_system(
    mut connections: Query<(&mut NetworkPeer, &mut UdpConnection)>,
    mut sockets: ResMut<SocketManager>,
) {
    // Accessing client data through a mutex is probably fine
    let connections = connections
        .iter_mut()
        .map(|(a,b)| {
            (b.address().clone(), Mutex::new((a, b)))
        })
        .collect::<BTreeMap<_, _>>();

    // Explicit borrows to prevent moves
    let connections = &connections;

    // Iterate over all sockets
    rayon::scope(|scope| {
        for socket in sockets.iter_sockets() {
            scope.spawn(move |_| {
                let socket = socket.socket();
                let mut buffer = [0u8; 1472];

                // Try to receive packets until we get an error
                loop {
                    let (len, origin) = match socket.recv_from(&mut buffer) {
                        Ok(success) => success,
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            break;
                        },
                        Err(e) => {
                            error!("Encountered an error when receiving from socket: {e}");
                            continue;
                        },
                    };

                    if let Some(mutex) = connections.get(&origin) {
                        // Block until we get access to the mutex lock.
                        // This is probably negligible, since most of the CPU time is syscalls and I/O
                        let lock = mutex.lock().unwrap();

                        todo!();
                    } else {
                        todo!();
                    }
                }
            });
        }
    });
}