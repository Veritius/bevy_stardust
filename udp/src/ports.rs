use std::{collections::BTreeMap, net::{UdpSocket, SocketAddr, Ipv4Addr, SocketAddrV4}};
use anyhow::{Result, bail};
use bevy::prelude::*;

#[derive(Debug, Resource)]
pub(crate) struct BoundSocketManager {
    pub sockets: BTreeMap<u16, BoundSocket>,
}

impl BoundSocketManager {
    pub fn new() -> Self {
        Self {
            sockets: BTreeMap::new(),
        }
    }

    /// Returns the least-populated bound socket.
    pub fn smallest(&self) -> u16 {
        let mut smallest_val: usize = usize::MAX;
        let mut smallest_idx: u16 = 0;

        for (k, v) in self.sockets.iter() {
            let len = v.clients.len();
            if len < smallest_val {
                smallest_val = len;
                smallest_idx = *k;
            }
        }

        smallest_idx
    }

    pub fn bind(&mut self, port: u16) -> Result<()> {
        let socket = UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)))?;
        socket.set_nonblocking(true)?;

        self.sockets.insert(port, BoundSocket::new(socket));
        Ok(())
    }

    pub fn unbind(&mut self, port: u16) -> Result<Vec<Entity>> {
        match self.sockets.remove(&port) {
            Some(val) => Ok(val.clients),
            None => bail!("Socket with port {port} was not present"),
        }
    }

    pub fn add_peer(&mut self, peer: Entity) {        
        // Check the peer isn't already added
        self.sockets.values()
        .for_each(|f| {
            if f.clients.contains(&peer) {
                todo!(); // Handle this case
            }
        });

        // Add to the least populated socket
        let smallest = self.smallest();
        self.sockets
            .get_mut(&smallest)
            .unwrap() // TODO: Handle this case
            .clients
            .push(peer);
    }
}

#[derive(Debug)]
pub(crate) struct BoundSocket {
    pub socket: UdpSocket,
    pub clients: Vec<Entity>,
}

impl BoundSocket {
    pub fn new(socket: UdpSocket) -> Self {
        BoundSocket {
            socket,
            clients: vec![],
        }
    }
}