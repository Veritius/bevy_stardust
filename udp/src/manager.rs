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
        address: impl ToSocketAddrs,
        listener: bool,
    ) -> Result<Entity> {
        todo!()
    }
}