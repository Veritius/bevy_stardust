use std::{net::SocketAddr, sync::Arc};
use bevy::ecs::{system::SystemParam, event::EventWriter};
use quinn_proto::{ServerConfig, ClientConfig};
use crate::events::EndpointManagerEvent;

/// API for controlling the QUIC endpoint.
#[derive(SystemParam)]
pub struct QuicManager<'w> {
    events: EventWriter<'w, EndpointManagerEvent>,
}

impl<'w> QuicManager<'w> {
    /// Opens the endpoint in server mode.
    pub fn start_server(&mut self, address: SocketAddr, capacity: u32, config: Arc<ServerConfig>) {
        self.events.send(EndpointManagerEvent::StartServer { address, capacity, config });
    }

    /// Opens the endpoint in client mode.
    pub fn start_client(&mut self, address: SocketAddr) {
        self.events.send(EndpointManagerEvent::StartClient { address });
    }

    /// Closes the endpoint.
    pub fn close_endpoint(&mut self) {
        self.events.send(EndpointManagerEvent::CloseEndpoint);
    }

    /// Try to connect to a server.
    pub fn try_connect(&mut self, address: SocketAddr, config: ClientConfig) {
        self.events.send(EndpointManagerEvent::TryConnect { address, config });
    }

    /// Allow new, incoming connections.
    pub fn allow_incoming(&mut self, value: bool) {
        self.events.send(EndpointManagerEvent::SetIncoming { allowed: value });
    }
}