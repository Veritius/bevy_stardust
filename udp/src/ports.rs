use std::{collections::BTreeMap, net::{UdpSocket, SocketAddr, Ipv4Addr, SocketAddrV4}};
use anyhow::{Result, bail};
use bevy::prelude::*;

#[derive(Debug, Resource)]
pub(crate) struct BoundSocketManager {
    sockets: BTreeMap<u16, BoundSocket>,
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
            let len = v.peers.len();
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
            Some(val) => Ok(val.peers),
            None => bail!("Socket with port {port} was not present"),
        }
    }

    pub fn add_peer(&mut self, peer: Entity) -> u16 {        
        // Check the peer isn't already added
        self.sockets.values()
        .for_each(|f| {
            if f.peers.contains(&peer) {
                todo!(); // Handle this case
            }
        });

        // Get the least populated socket in the map
        let smallest = self.smallest();
        let socket = self.sockets
            .get_mut(&smallest)
            .unwrap(); // TODO: Handle this case
        
        // Add to the least populated socket
        socket.peers.push(peer);
        socket.peers.sort_unstable();

        // Return the port they were assigned to
        return smallest;
    }

    pub fn iter(&self) -> impl Iterator<Item = (u16, &BoundSocket)> {
        self.sockets.iter().map(|(k,v)| (*k, v))
    }
}

#[derive(Debug)]
pub(crate) struct BoundSocket {
    pub socket: UdpSocket,
    /// Peers associated with this socket. This list is sorted.
    pub peers: Vec<Entity>,
}

impl BoundSocket {
    pub fn new(socket: UdpSocket) -> Self {
        BoundSocket {
            socket,
            peers: vec![],
        }
    }
}