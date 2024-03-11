use std::net::ToSocketAddrs;
use anyhow::Result;
use bevy_ecs::{prelude::*, system::SystemParam};
use crate::{endpoint::Endpoint, connection::Connection};

/// A SystemParam that lets you create [`Endpoints`](Endpoint) and open outgoing [`Connections`](Connection).
#[derive(SystemParam)]
pub struct UdpManager<'w, 's> {
    commands: Commands<'w, 's>,
    endpoints: Query<'w, 's, &'static mut Endpoint>,
    connections: Query<'w, 's, &'static mut Connection>,
}

impl UdpManager<'_, '_> {
    /// Opens an endpoint bound to the address returned by [`to_socket_addrs`](ToSocketAddrs::to_socket_addrs).
    pub fn open_endpoint(
        &mut self,
        address: impl ToSocketAddrs,
        listener: bool,
    ) -> Result<Entity> {
        todo!()
    }

    /// Opens a connection to `address` routed through `endpoint`.
    pub fn open_connection(
        &mut self,
        address: impl ToSocketAddrs,
        endpoint: Entity,
    ) -> Result<Entity> {
        todo!()
    }

    /// Opens an endpoint and tries to connect to a remote peer from it.
    pub fn open_endpoint_and_connect(
        &mut self,
        address: impl ToSocketAddrs,
        remote: impl ToSocketAddrs,
    ) -> Result<Entity> {
        todo!()
    }
}