use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use super::UdpTransportState;
use super::connections::{PendingUdpPeer, PendingDirection};
use super::ports::PortBindings;

/// Manages the UDP transport layer.
#[derive(SystemParam)]
pub struct UdpConnectionManager<'w, 's> {
    commands: Commands<'w, 's>,
    state: Res<'w, State<UdpTransportState>>,
    ports: Option<ResMut<'w, PortBindings>>,
}

impl<'w, 's> UdpConnectionManager<'w, 's> {
    /// Binds to a set of ports, allowing connections to be established.
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
        // self.commands.insert_resource(ManagerAction::Shutdown);
        todo!()
    }

    /// Enables listening for new remote connections.
    /// This is useful in a client/server architecture where you want to act as a server.
    pub fn enable_listening(&mut self) {
        todo!()
    }

    /// Try to connect to a remote peer.
    /// 
    /// `address` is the address that will be used to start the connection. The actual address used during an active connection may change.
    /// `timeout` is how long the transport layer will try to establish a connection before giving up. If `None`, defaults to 30 seconds.
    pub fn connect_to_remote(&mut self, address: SocketAddr, timeout: Option<Duration>) {
        // Check the state and warn if incorrect
        if *self.state.get() == UdpTransportState::Offline {
            warn!("Couldn't connect to remote server: transport layer is offline");
            return;
        }

        // Create entity to store connection attempt
        let pending = self.commands.spawn((
            PendingUdpPeer {
                address,
                started: Instant::now(),
                timeout: timeout.unwrap_or(Duration::from_secs(30)),
                direction: PendingDirection::Outgoing(super::connections::PendingOutgoingState::NoResponseYet),
            },
        )).id();

        // If the state isn't Offline then this resource exists, so we can do this
        self.ports.as_mut().unwrap().add_client(pending);
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

/// Puts the transport layer into standby immediately without needing to go through a tick.
/// Intended for use in headless applications, ie dedicated servers. Also used in the examples.
/// 
/// Panics if necessary plugins haven't been added, the transport layer is already in standby, or there's a problem starting up (binding ports, etc)
pub fn startup_now(world: &mut World, address: Option<IpAddr>, ports: &[u16]) {
    let address = if address.is_some() { address.unwrap() } else { IpAddr::V4(Ipv4Addr::UNSPECIFIED) };
    let mut ports = ports.iter().map(|f| f.clone()).collect::<Vec<_>>();
    ports.sort_unstable();
    ports.dedup();
    world.insert_resource(PortBindings::new(address, &ports).unwrap());
    world.resource_mut::<NextState<UdpTransportState>>().set(UdpTransportState::Active);
    apply_state_transition::<UdpTransportState>(world);
    info!("Immediately started multiplayer with {address}:{ports:?}");
}