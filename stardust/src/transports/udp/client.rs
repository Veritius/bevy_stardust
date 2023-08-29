use std::net::SocketAddr;
use bevy::{prelude::*, ecs::system::SystemParam};
use crate::transports::udp::{UdpTransportState, UdpConnectionError};

/// Interface for using the UDP transport layer in client mode.
#[derive(SystemParam)]
pub struct UdpClientManager<'w, 's> {
    commands: Commands<'w, 's>,
    state: Res<'w, State<UdpTransportState>>,
    next: ResMut<'w, NextState<UdpTransportState>>,
}

impl UdpClientManager<'_, '_> {
    /// Try to connect to a remote server.
    pub fn connect(&mut self, address: SocketAddr) -> Result<(), UdpConnectionError> {
        // Check state
        if *self.state.get() != UdpTransportState::Disabled {
            return match self.state.get() {
                UdpTransportState::Client => Err(UdpConnectionError::ClientExists),
                UdpTransportState::Server => Err(UdpConnectionError::ServerExists),
                _ => panic!()
            }
        }

        // Check next state
        match self.next.0 {
            Some(value) => {
                return match value {
                    UdpTransportState::Client => Err(UdpConnectionError::ClientExists),
                    UdpTransportState::Server => Err(UdpConnectionError::ServerExists),
                    UdpTransportState::Disabled => panic!(),
                }
            },
            None => {},
        }

        todo!();

        Ok(())
    }

    /// Disconnect from a remote server if connected to one.
    pub fn disconnect(&mut self) {
        todo!()
    }
}