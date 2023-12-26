use std::sync::Exclusive;
use bevy::prelude::*;
use crate::events::EndpointManagerEvent;

// Sync wrapper for Endpoint
#[derive(Resource, Default)]
pub(crate) enum Endpoint {
    #[default]
    Closed,

    Open(Exclusive<quinn_proto::Endpoint>),
}

pub(super) fn endpoint_manager_system(
    mut endpoint: ResMut<Endpoint>,
    mut events: EventReader<EndpointManagerEvent>,
) {
    for event in events.read() {
        match event {
            EndpointManagerEvent::StartServer { address, capacity, config } => {
                todo!()
            },
            EndpointManagerEvent::StartClient { address } => {
                todo!()
            },
            EndpointManagerEvent::CloseEndpoint => {
                todo!()
            },
            EndpointManagerEvent::TryConnect { address, config } => {
                todo!()
            },
            EndpointManagerEvent::SetIncoming { allowed } => {
                todo!()
            },
        }
    }
}