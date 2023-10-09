use std::net::{IpAddr, Ipv4Addr};
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use super::UdpTransportState;
use super::ports::PortBindings;

/// Manages the UDP transport layer.
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
    /// `ports` is the set of ports that will be used for connection purposes. There must be at least one value passed. More values will improve parallelism, to a point.
    /// In almost all cases, the amount of values passed should be at most the amount of logical cores on the system OR the amount of peers you expect to be connected at any one time, whichever is lesser.
    pub fn startup(&mut self, address: Option<IpAddr>, ports: &[u16]) {
        let address = if address.is_some() { address.unwrap() } else { IpAddr::V4(Ipv4Addr::UNSPECIFIED) };
        let mut ports = ports.iter().map(|f| f.clone()).collect::<Vec<_>>();
        ports.sort_unstable();
        ports.dedup();
        self.commands.insert_resource(ManagerAction::Startup { address, ports });
    }

    /// Inform all peers of shutdown and disconnect all ports.
    pub fn shutdown(&mut self) {
        self.commands.insert_resource(ManagerAction::Shutdown);
    }
}

#[derive(Debug, Clone, Resource)]
pub(super) enum ManagerAction {
    Startup {
        address: IpAddr,
        ports: Vec<u16>,
    },
    Shutdown,
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
        ManagerAction::Startup { address, ports } => {
            // Check state
            if *state.get() != UdpTransportState::Offline {
                info!("Didn't start multiplayer: already started");
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
            next_state.set(UdpTransportState::Active);
        },
        ManagerAction::Shutdown => {
            info!("Shut down multiplayer");
            commands.remove_resource::<PortBindings>();
            next_state.set(UdpTransportState::Offline);
        },
    }
}