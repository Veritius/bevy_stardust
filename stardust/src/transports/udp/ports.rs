//! Dynamic port binding system.

use std::{net::{UdpSocket, IpAddr}, collections::BTreeMap, sync::Mutex};
use bevy::prelude::{Entity, Resource};
use anyhow::{Result, bail};

/// Owns UdpSockets and associates Entity ids with them.
#[derive(Debug, Resource)]
pub(super) struct PortBindings {
    sockets: BTreeMap<u16, BoundUdpSocket>,
    reservations: Mutex<Vec<(Entity, u16)>>,
}

impl PortBindings {
    /// Makes a new `PortBindings`, creating `UdpSocket`s bound to all in `ports`
    pub fn new(addr: IpAddr, ports: &[u16]) -> Result<Self> {
        // Check range of ports is acceptable
        if ports.len() == 0 {
            bail!("Ports slice had a size of zero");
        }

        // Create manager
        let mut mgr = Self {
            sockets: BTreeMap::new(),
            reservations: Mutex::default(),
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

    /// Returns a bound port by its port number.
    pub fn port(&self, port: u16) -> Option<(&UdpSocket, &[Entity])> {
        let socket = self.sockets.get(&port);
        if socket.is_none() { return None; }
        let socket = socket.unwrap();
        Some((&socket.socket, &socket.clients))
    }

    /// Associates a client with a bound socket, then returns the port.
    pub fn add_client(&mut self, client: Entity) -> u16 {
        let port = self.least_filled_port();
        self.sockets.get_mut(&port).unwrap().clients.push(client);
        port
    }

    /// Disassociates a client from its bound socket, if present.
    pub fn remove_client(&mut self, client: Entity) {
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

    pub fn make_reservation(&self, id: Entity) -> u16 {
        // TODO: This locks the mutex twice, fix this
        let port = self.least_filled_port();
        self.reservations.lock().unwrap().push((id, port));
        return port
    }

    /// Commits the reservations to the BoundUdpSockets.
    pub fn commit_reservations(&mut self) {
        let mut reservations = self.reservations.lock().unwrap();
        for (id, port) in reservations.drain(..) {
            self.sockets.get_mut(&port).unwrap().clients.push(id);
        }
    }

    /// Returns the least occupied port, including reservations.
    fn least_filled_port(&self) -> u16 {
        let mut counter: Vec<(u16, usize)> = vec![];

        // Add all active to counter
        for (port, socket) in self.sockets.iter() {
            counter.push((*port, socket.clients.len()));
        }

        // Count reservations
        let reservations = self.reservations.lock().unwrap();
        for (_, port) in &*reservations {
            for (skt, count) in counter.iter_mut() {
                if *port == *skt {
                    *count += 1;
                    break
                }
            }
        }

        // Find the smallest value
        let mut smallest = (u16::MIN, usize::MAX);
        for (port, value) in counter.iter() {
            if *value < smallest.1 { smallest.0 = *port }
        }

        // Return the smallest value
        smallest.0
    }
}

#[derive(Debug)]
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