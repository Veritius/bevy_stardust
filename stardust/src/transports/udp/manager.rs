use anyhow::{Result, bail, Context};
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use super::{StateChangeBlocker, UdpTransportState, ports::PortBindings};

/// Manages the UDP transport layer.
#[derive(SystemParam)]
pub struct UdpConnectionManager<'w, 's> {
    state: Res<'w, State<UdpTransportState>>,
    blocker: ResMut<'w, StateChangeBlocker>,
    next_state: ResMut<'w, NextState<UdpTransportState>>,
    commands: Commands<'w, 's>,
}

impl<'w, 's> UdpConnectionManager<'w, 's> {
    /// Binds to a set of ports and sets the transport layer to standby.
    /// To actually start connecting, use `start_server` or `start_client`.
    /// 
    /// `address` is the IP address that the transport layer will try to use.
    /// A value of `Some` will ask the OS to use that IP specifically, and a value of `None` will let the OS choose.
    /// This IP is only within the local area network, and does not affect your remote IP, if connected to the Internet.
    /// 
    /// `ports` is the set of ports that will be used for connection purposes. There must be at least one value passed.
    /// More values will improve parallelism, to a point. In almost all cases, the amount of values passed should be at most the amount of logical cores on the system.
    /// Additionally, if acting as a client, it's best to allocate only one port.
    pub fn start_multiplayer(&mut self, address: Option<IpAddr>, ports: &[u16]) -> Result<()> {
        // Check we're in the right state to do this
        if *self.state.get() != UdpTransportState::Offline {
            bail!("can only start multiplayer when offline");
        }

        // Check if we're blocked by something
        if self.blocker.blocked() { bail!("blocked: {}", *self.blocker); }

        // Check ports slice length
        if ports.len() == 0 { bail!("ports slice must have at least 1 item"); }

        // Bind ports
        let ip_addr = if address.is_none() { IpAddr::V4(Ipv4Addr::UNSPECIFIED) } else { address.unwrap() };
        let bindings = PortBindings::new(ip_addr, ports);
        if bindings.is_err() { bail!("failed to bind ports: {}", bindings.unwrap_err()); }
        self.commands.insert_resource(bindings.unwrap());

        // All good
        self.next_state.set(UdpTransportState::Standby);
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

        // Mark as blocked
        *self.blocker = StateChangeBlocker::StartingClient;

        // All good
        return Ok(())
    }

    /// Stop the client, informing the remote server if one is present, and return to standby.
    /// If there is nothing to disconnect from, this function will do nothing.
    pub fn client_disconnect(&mut self) {
        // Check we're in the right conditions to do this
        if *self.state.get() != UdpTransportState::Client { return; }
        if self.blocker.blocked() { return; }

        // Mark as blocked
        *self.blocker = StateChangeBlocker::StoppingClient;
    }

    /// Start listening for connections as a server.
    pub fn start_server(&mut self) -> Result<()> {
        // Check we're in the right state to do this
        if *self.state.get() != UdpTransportState::Standby {
            bail!("can only start a server when in standby");
        }

        // Check if we're blocked by something
        if self.blocker.blocked() { bail!("blocked: {}", *self.blocker); }

        // Mark as blocked
        *self.blocker = StateChangeBlocker::StartingServer;

        // All good
        self.next_state.set(UdpTransportState::Server);
        return Ok(())
    }

    /// Stop the server, informing clients of the disconnection, and return to standby.
    /// If there is no server to stop, this function will do nothing.
    pub fn stop_server(&mut self) {
        // Check we're in the right conditions to do this
        if *self.state.get() != UdpTransportState::Server { return; }
        if self.blocker.blocked() { return; }

        // Mark as blocked
        *self.blocker = StateChangeBlocker::StoppingServer;
    }

    /// Closes active connections and disconnects from any bound ports.
    pub fn stop_multiplayer(&mut self) {
        todo!()
    }
}