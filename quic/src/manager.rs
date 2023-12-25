use std::net::SocketAddr;
use bevy::ecs::{system::SystemParam, event::EventWriter};
use crate::events::*;

/// API for controlling the QUIC endpoint.
#[derive(SystemParam)]
pub struct QuicManager<'w> {
    start_evs: EventWriter<'w, StartEndpointEvent>,
    close_evs: EventWriter<'w, CloseEndpointEvent>,
    connect_evs: EventWriter<'w, TryConnectEvent>,
    incoming_evs: EventWriter<'w, ToggleIncomingEvent>,
}

impl<'w> QuicManager<'w> {
    /// Opens the endpoint.
    pub fn start_endpoint(&mut self, address: SocketAddr) {
        self.start_evs.send(StartEndpointEvent {
            address,
        });
    }

    /// Closes the endpoint.
    pub fn close_endpoint(&mut self) {
        self.close_evs.send(CloseEndpointEvent);
    }

    /// Try to connect to an endpoint as a server.
    pub fn try_connect(&mut self, target: SocketAddr) {
        self.connect_evs.send(TryConnectEvent {
            target,
        });
    }

    /// Allow new, incoming connections.
    pub fn allow_incoming(&mut self, value: bool) {
        self.incoming_evs.send(ToggleIncomingEvent(value));
    }
}