use std::net::SocketAddr;
use bevy::prelude::*;

#[derive(Event)]
pub(crate) struct StartEndpointEvent {
    pub address: SocketAddr,
}

#[derive(Event)]
pub(crate) struct CloseEndpointEvent;

#[derive(Event)]
pub(crate) struct TryConnectEvent {
    pub target: SocketAddr,
}

#[derive(Event)]
pub(crate) struct ToggleIncomingEvent(pub bool);