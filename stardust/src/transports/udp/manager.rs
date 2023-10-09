use std::marker::PhantomData;
use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use bevy::prelude::*;
use bevy::ecs::system::{SystemParam, SystemBuffer};

use super::UdpTransportState;
use super::ports::PortBindings;

/// Manages the UDP transport layer.
/// 
/// Actions applied with this systemparam are deferred, and applied at `PostUpdate`.
/// When multiple actions are applied in different systems, the result will be whatever is processed last, ie non-deterministic.
#[derive(SystemParam)]
pub struct UdpConnectionManager<'w, 's> {
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
    pub fn start_multiplayer(&mut self, address: Option<IpAddr>, ports: &[u16]) {
        let address = if address.is_some() { address.unwrap() } else { IpAddr::V4(Ipv4Addr::UNSPECIFIED) };
        let ports = ports.iter().map(|f| f.clone()).collect::<Vec<_>>();
        self.commands.insert_resource(ManagerAction::StartMultiplayer { address, ports });
    }

    /// Closes active connections and disconnects from any bound ports.
    pub fn stop_multiplayer(&mut self) {
        self.commands.insert_resource(ManagerAction::StopMultiplayer);
    }

    /// Try to connect to `remote` as a client.
    pub fn start_client(&mut self, remote: SocketAddr) {
        self.commands.insert_resource(ManagerAction::StartClient { remote });
    }

    /// Stop the client, informing the remote server if one is present, and return to standby.
    /// If there is nothing to disconnect from, this function will do nothing.
    pub fn stop_client(&mut self) {
        self.commands.insert_resource(ManagerAction::StopClient);
    }

    /// Start listening for connections as a server.
    pub fn start_server(&mut self) {
        self.commands.insert_resource(ManagerAction::StartServer);
    }

    /// Stop the server, informing clients of the disconnection, and return to standby.
    /// If there is no server to stop, this function will do nothing.
    pub fn stop_server(&mut self) {
        self.commands.insert_resource(ManagerAction::StopServer);
    }
}

#[derive(Debug, Clone, Resource)]
pub(super) enum ManagerAction {
    StartMultiplayer {
        address: IpAddr,
        ports: Vec<u16>,
    },
    StopMultiplayer,
    StartClient {
        remote: SocketAddr,
    },
    StopClient,
    StartServer,
    StopServer,
}

pub(super) fn apply_manager_action_system(
    mut commands: Commands,
    state: Res<State<UdpTransportState>>,
    mut next_state: ResMut<NextState<UdpTransportState>>,
    action: Option<Res<ManagerAction>>,
) {
    if action.is_none() { return; }
    commands.remove_resource::<ManagerAction>();

    match action.unwrap().clone() {
        ManagerAction::StartMultiplayer { address, ports } => {
            // Check state
            if *state.get() != UdpTransportState::Offline {
                info!("Failed to start multiplayer: already started");
                return;
            }

            // Bind ports and check if the OS said no
            match PortBindings::new(address, &ports) {
                Ok(ports) => {
                    commands.insert_resource(ports);
                },
                Err(err) => {
                    error!("Failed to start multiplayer: {}", err);
                    return;
                },
            }

            info!("Started multiplayer with {address}:{ports:?}");
            next_state.set(UdpTransportState::Standby);
        },
        ManagerAction::StopMultiplayer => {
            // Check state
            if *state.get() == UdpTransportState::Offline {
                info!("Failed to stop multiplayer: already stopped");
                return;
            }

            // OS will close the ports for us
            commands.remove_resource::<PortBindings>();
            next_state.set(UdpTransportState::Offline);
        },
        ManagerAction::StartClient { remote } => todo!(),
        ManagerAction::StopClient => todo!(),
        ManagerAction::StartServer => todo!(),
        ManagerAction::StopServer => todo!(),
    }
}