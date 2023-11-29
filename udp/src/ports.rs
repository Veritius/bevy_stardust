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