use std::net::IpAddr;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::transports::udp::{UdpTransportMode, ProcessingMode};
use super::{listener::UdpListener, ports::PortBindings};

/// Interface for using the UDP transport layer in server mode.
#[derive(SystemParam)]
pub struct UdpServerManager<'w, 's> {
    commands: Commands<'w, 's>,
    processing_mode: Res<'w, ProcessingMode>,
    state: Res<'w, State<UdpTransportMode>>,
    next: ResMut<'w, NextState<UdpTransportMode>>,
}

impl UdpServerManager<'_, '_> {
    /// Starts hosting a server based on `config`.
    pub fn start_server(&mut self, config: ServerConfig) -> Result<(), UdpServerError> {
        // Check state
        if *self.state.get() != UdpTransportMode::Disabled {
            return match self.state.get() {
                UdpTransportMode::Client => Err(UdpServerError::RunningAsClient),
                UdpTransportMode::Server => Err(UdpServerError::RunningAsServer),
                _ => panic!()
            }
        }

        // Get address
        let address = config.address.unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));

        // Check listen port
        if config.active_ports.contains(&config.listen_port) {
            return Err(UdpServerError::ListenPortInActivePorts)
        }

        // Create listener
        let listener = match UdpListener::new(address, config.listen_port) {
            Ok(value) => value,
            Err(error) => { return Err(UdpServerError::IoError(error.kind())) },
        };

        // Check if we're in single processing mode
        if *self.processing_mode == ProcessingMode::Single {
            self.commands.insert_resource(listener);
            self.next.set(UdpTransportMode::Server);
            return Ok(())
        }

        // Deduplicate active ports set and bind to them
        let mut ports = config.active_ports.clone();
        ports.sort_unstable(); ports.dedup();
        if ports.len() == 0 { return Err(UdpServerError::ActivePortsEmpty); }
        let bindings = match PortBindings::new(address, &ports) {
            Ok(values) => values,
            Err(error) => { return Err(UdpServerError::IoError(error.kind())) },
        };

        // Add resources
        self.commands.insert_resource(listener);
        self.commands.insert_resource(bindings);

        // Change state
        self.next.set(UdpTransportMode::Server);

        // Everything worked
        Ok(())
    }

    /// Stops hosting a server, disconnecting all clients.
    pub fn stop_server(&mut self) {
        todo!()
    }

    /// Allows/prevents new clients from joining.
    pub fn prevent_connections(&mut self, _value: bool) {
        todo!()
    }
}

/// Configuration for running a server.
#[derive(Debug)]
pub struct ServerConfig {
    /// The address to use to connect. Use `None` if you want the OS to allocate one for you.
    /// 
    /// Note: This is the local address within your system, and will not be the IP used by clients to connect over the Internet.
    /// That value is usually assigned by your ISP, and you can quickly see it by viewing this website: https://icanhazip.com/
    pub address: Option<IpAddr>,

    /// The port that will be used by new clients to join the game.
    /// 
    /// If `ProcessingMode` is `Single`, this will also be the active port.
    pub listen_port: u16,

    /// The ports that will be used in the dynamic port allocator system.
    /// This is not used in `ProcessingMode::Single`, and you can leave it empty if so.
    /// 
    /// Higher values improve how well the server scales to player count, to an extent.
    /// The highest you should set this is the amount of logical CPU cores that are on the system.
    pub active_ports: Vec<u16>,
}

/// An error related to the UDP server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UdpServerError {
    /// Some kind of IO-related error.
    IoError(std::io::ErrorKind),
    /// Tried to start the server, but layer is in client mode.
    RunningAsClient,
    /// Tried to start the server, but it was already running.
    RunningAsServer,
    /// The listen port was in the active ports list.
    ListenPortInActivePorts,
    /// The list of active ports was empty.
    ActivePortsEmpty,
}