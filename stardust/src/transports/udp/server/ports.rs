use std::{net::{UdpSocket, IpAddr}, collections::BTreeMap};
use bevy::prelude::{Entity, Resource, info};

/// Owns UdpSockets and associates Entity ids with them.
#[derive(Resource)]
pub(super) struct PortBindings {
    sockets: BTreeMap<u16, BoundUdpSocket>,
}

impl PortBindings {
    /// Makes a new `PortBindings`, creating `UdpSocket`s bound to all in `ports`
    pub fn new(addr: IpAddr, ports: &[u16]) -> Result<Self, std::io::Error> {
        // Check range of ports is acceptable
        if ports.len() == 0 {
            panic!("Amount of ports used must be at least one");
        }

        // Log port connections
        info!("Bound to ports {:?}, total of {}", ports, ports.len());

        // Create manager
        let mut mgr = Self {
            sockets: BTreeMap::new(),
        };

        // Create ports from range
        for port in ports {
            let port = *port;
            let socket = UdpSocket::bind((addr, port))?;
            socket.set_nonblocking(true)?;
            mgr.sockets.insert(port, BoundUdpSocket::new(socket));
        }

        Ok(mgr) // the only thing i know for real
    }

    /// Returns an iterator of all bound ports and their associated clients.
    pub fn iter(&self) -> impl Iterator<Item = (u16, &UdpSocket, &[Entity])> {
        self.sockets
            .iter()
            .map(|(port, bound)| (*port, &bound.socket, bound.clients.as_slice()))
    }

    /// Associates a client with a bound socket, then returns the port.
    pub fn add_client(&mut self, client: Entity) -> u16 {
        // Find the first smallest vec
        let mut last_len: usize = 0;
        for (port, socket) in self.sockets.iter_mut() {
            let len = socket.clients.len();
            if len < last_len {
                socket.clients.push(client);
                return *port;
            }
            last_len = len;
        }

        // Add to first vec if the last try failed
        if last_len == 0 {
            let mut socket = self.sockets
                .first_entry()
                .unwrap();
            socket.get_mut().clients.push(client);
            return *socket.key();
        }

        // This should never happen
        panic!("Failed to add a client to a BoundUdpSocket. This should never happen. I suggest improving your computer's protection against cosmic rays.");
    }

    /// Disassociates a client from its bound socket, if present.
    pub fn _remove_client(&mut self, client: Entity) {
        for bound in self.sockets.values_mut() {
            let mut bound_iter = bound.clients.iter().enumerate();
            let target = loop {
                let Some((index, ent)) = bound_iter.next() else { break None; };
                if *ent != client { continue }
                break Some(index);
            };

            let Some(target) = target else { continue };
            bound.clients.remove(target);
            return;
        }
    }
}

struct BoundUdpSocket {
    pub socket: UdpSocket,
    pub clients: Vec<Entity>,
}

impl BoundUdpSocket {
    fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            clients: vec![],
        }
    }
}