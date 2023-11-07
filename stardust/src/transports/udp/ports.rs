//! Dynamic port binding system.

use std::{net::{UdpSocket, IpAddr}, collections::BTreeMap, sync::Mutex};
use bevy::prelude::{Entity, Resource};
use anyhow::{Result, bail};
use smallvec::{smallvec, SmallVec};

/// Owns UdpSockets and associates Entity ids with them.
#[derive(Debug, Resource)]
pub(super) struct PortBindings {
    sockets: BTreeMap<u16, BoundUdpSocket>,
    reserved: Mutex<(usize, BTreeMap<u16, Vec<usize>>)>,
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
            reserved: Mutex::new((0, BTreeMap::new())),
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

    /// Reserves a position in the port bindings without having to provide an entity id.
    /// Eventually, `take_reservation` must be used to fulfil the reservation.
    pub fn make_reservation(&self) -> (u16, ReservationKey) {
        let least = self.least_filled_port();
        let mut lock = self.reserved.lock().unwrap();
        let key = ReservationKey(lock.0); lock.0 += 1;
        lock.1.entry(least).or_insert(vec![]).push(key.0);
        (least, key)
    }

    /// Fulfils reservations.
    pub fn take_reservations(&mut self, iter: impl Iterator<Item = (ReservationKey, Entity)>) {
        let mut lock = self.reserved.lock().unwrap();
        'outer: for (key, id) in iter {
            for (k, v) in lock.1.iter_mut() {
                let mut iter = v.iter().enumerate();
                let z = 'items: loop {
                    let (i, j) = if let Some(g) = iter.next() { (g.0, g.1) } else { continue 'outer };
                    if *j == key.0 { break 'items i }
                };
                v.remove(z);
                self.sockets.get_mut(k).unwrap().clients.push(id);
                continue 'outer;
            }
        }
    }

    /// Panics if there are any pending reservations.
    #[cfg(debug_assertions="true")]
    pub fn confirm_reservation_emptiness(&self) {
        self.reserved.lock().unwrap().1.values().for_each(|f| if f.len() > 0 { panic!() })
    }

    /// Returns the least occupied port, including reservations.
    fn least_filled_port(&self) -> u16 {
        let mut counter: SmallVec<[(u16, usize); 8]> = smallvec![];

        // Add all active to counter
        for (port, socket) in self.sockets.iter() {
            counter.push((*port, socket.clients.len()));
        }

        // Add reservations to counter
        'reserved: for (port, reserved) in self.reserved.lock().unwrap().1.iter() {
            for (c_port, val) in counter.iter_mut() {
                if *c_port == *port {
                    *val += reserved.len();
                    continue 'reserved;
                }
            }
            counter.push((*port, reserved.len()));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservationKey(usize);

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