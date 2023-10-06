use anyhow::{Result, bail};
use std::net::{SocketAddr, IpAddr};
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;

use super::{NetActionBlocker, UdpTransportState};

/// Manages the UDP transport layer.
#[derive(SystemParam)]
pub struct UdpConnectionManager<'w, 's> {
    commands: Commands<'w, 's>,
    state: Res<'w, State<UdpTransportState>>,
    blocker: ResMut<'w, NetActionBlocker>,
}

impl<'w, 's> UdpConnectionManager<'w, 's> {
    /// Binds to a set of ports and sets the transport layer to standby.
    /// To actually start connecting, use `start_server` or `start_client`.
    /// 
    /// `address` is the IP address that the transport layer will try to use.
    /// A value of `Some` will ask the OS to use that IP specifically, and a value of `None` will let the OS choose.
    /// This IP is only within the local area network, and does not affect your remote IP, if connected to the Internet.
    /// 
    /// `ports` is the set of ports that will be used for connection purposes.
    /// There must always be at least one value in the passed set.
    /// If you are using `ProcessingMode::Single` this should only have one value, otherwise unnecessary ports will be bound.
    /// 
    /// If you are using `ProcessingMode::Taskpool`, you can pass multiple values, with higher amounts of ports improving parallel performance.
    /// The highest you should set this is the number of logical cores on your system, but you can allocate less if needed.
    /// Values that are higher than the number of logical cores on your system will not give any extra parallelism benefits.
    pub fn start_multiplayer(&mut self, address: Option<SocketAddr>, ports: &[u16]) -> Result<()> {
        // Check we're in the right state to do this
        if *self.state.get() != UdpTransportState::Offline {
            bail!("can only start multiplayer when offline");
        }

        // Check if we're blocked by something
        if self.blocker.blocked() { bail!("blocked: {}", *self.blocker); }

        // Check ports slice length
        if ports.len() == 0 { bail!("ports slice must have at least 1 item"); }

        // All good
        return Ok(())
    }

    /// Try to connect to `remote` as a client.
    pub fn start_client(&mut self, remote: SocketAddr) -> Result<()> {
        // Check we're in the right state to do this
        if *self.state.get() != UdpTransportState::Standby {
            bail!("can only start a client when in standby");
        }

        // Check if we're blocked by something
        if self.blocker.blocked() { bail!("blocked: {}", *self.blocker); }

        // All good
        return Ok(())
    }

    /// Stop the client, informing the remote server if one is present, and return to standby.
    /// If there is nothing to disconnect from, this function will do nothing.
    pub fn client_disconnect(&mut self) {
        
    }

    /// Start listening for connections as a server.
    pub fn start_server(&mut self) -> Result<()> {
        // Check we're in the right state to do this
        if *self.state.get() != UdpTransportState::Standby {
            bail!("can only start a server when in standby");
        }

        // Check if we're blocked by something
        if self.blocker.blocked() { bail!("blocked: {}", *self.blocker); }

        // All good
        return Ok(())
    }

    /// Stop the server, informing clients of the disconnection, and return to standby.
    /// If there is no server to stop, this function will do nothing.
    pub fn stop_server(&mut self) {

    }

    /// Closes active connections and disconnects from any bound ports.
    pub fn stop_multiplayer(&mut self) {
        todo!()
    }
}