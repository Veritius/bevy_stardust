use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::{ConnectionState, EndpointWeak};

/// Manages endpoints when used as a `Resource` in the world.
/// 
/// Deals with events, new connections, etc.
/// 
/// Requires [`QuicPlugin`](crate::QuicPlugin) or [`EndpointHandler::system`] to be added to be functional.
/// 
/// This type does not accept strong handles to endpoints - you still have to manage those by yourself.
/// Endpoint handles that have been dropped are removed automatically.
#[derive(Resource)]
pub struct EndpointHandler {
    set: Vec<EndpointWeak>,
}

impl EndpointHandler {
    /// Creates a new, empty `EndpointHandler`.
    pub fn new() -> Self {
        Self {
            set: Vec::new(),
        }
    }

    /// Adds a new endpoint to the handler.
    pub fn insert(&mut self, handle: EndpointWeak) {
        // If we already have the handle in our collection, don't insert it as it'd waste resources
        if self.set.iter().find(|v| *v == &handle).is_some() { return }
        self.set.push(handle);
    }

    /// Removes an endpoint from the handler if present.
    pub fn remove(&mut self, handle: &EndpointWeak) {
        let idx = self.set.iter()
            .enumerate()
            .find(|(_, v)| *v == handle)
            .map(|(v,_)| v);

        if let Some(idx) = idx {
            self.set.remove(idx);
        }
    }

    /// A Bevy system for operating the `EndpointHandler`.
    pub fn system(
        mut commands: Commands,
        mut handler: ResMut<EndpointHandler>,
        mut connecting_events: EventWriter<PeerConnectingEvent>,
        mut connected_events: EventWriter<PeerConnectedEvent>,
    ) {
        let mut removals = Vec::new();
        for (index, handle) in handler.set.iter().enumerate() {
            match handle.clone().upgrade() {
                Some(handle) => {
                    if let Some(connection) = handle.poll_connections() {
                        let state = connection.state();

                        match state {
                            ConnectionState::Closing | ConnectionState::Closed => { return },
                            _ => {},
                        }

                        let id = commands.spawn((
                            connection,

                            Peer::new(),
                            PeerMessages::<Incoming>::new(),
                            PeerMessages::<Outgoing>::new(),

                            match state {
                                ConnectionState::Connecting => PeerLifestage::Handshaking,
                                ConnectionState::Connected => PeerLifestage::Established,
                                ConnectionState::Closing => PeerLifestage::Closing,
                                ConnectionState::Closed => PeerLifestage::Closed,
                            },
                        )).id();

                        match state {
                            ConnectionState::Connecting => { connecting_events.send(PeerConnectingEvent { peer: id }); },
                            ConnectionState::Connected => { connected_events.send(PeerConnectedEvent { peer: id }); },
                            ConnectionState::Closing | ConnectionState::Closed => unreachable!(),
                        };
                    }
                },
    
                None => { removals.push(index); },
            }
        }
    
        for index in removals.iter().rev() {
            handler.set.remove(*index);
        }
    }
}