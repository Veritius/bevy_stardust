use bevy::{prelude::*, ecs::system::SystemParam};
use crate::{sockets::SocketManagerEvent, config::ConnectionConfig};

#[derive(SystemParam)]
pub struct UdpConnectionManager<'w> {
    socket_manager: EventWriter<'w, SocketManagerEvent>,
    config: ResMut<'w, ConnectionConfig>,
}

impl<'w> UdpConnectionManager<'w> {
    /// Sets whether new incoming connections are allowed or not.
    /// This will always enable outgoing connections requested by this peer.
    pub fn allow_incoming_connections(&mut self, value: bool) {
        self.config.allow_incoming_connections = value;
    }

    /// Sets the maximum number of connections.
    /// New outgoing or incoming connections will fail if the total would be more than the limit.
    /// Existing connections are unaffected, even if there are more than the limit.
    pub fn set_maximum_connections(&mut self, value: u32) {
        self.config.maximum_active_connections = value;
    }

    /// Closes all sockets, closing all connections.
    pub fn close_sockets(&mut self) {
        self.socket_manager.send(SocketManagerEvent::ClearSockets { disconnect: true });
    }
}