use std::mem;
use bevy_ecs::prelude::*;
use bevy_stardust::prelude::*;
use crate::{endpoint::LoadingEndpoint, ConnectionState, Endpoint, EndpointError, EndpointWeak};

/// Waits for endpoints to finish.
pub struct WaitingEndpoints {
    set: Vec<LoadingEndpoint>,
    swap: Vec<LoadingEndpoint>,
}

impl WaitingEndpoints {
    /// Returns a new, empty `WaitingEndpoints` struct.
    pub fn new() -> Self {
        Self {
            set: Vec::new(),
            swap: Vec::new(),
        }
    }

    /// Adds `future` to the queue of endpoints being checked.
    pub fn insert(&mut self, future: LoadingEndpoint) {
        self.set.push(future);
    }

    /// Returns the next endpoint that's done processing, if any
    pub fn poll(&mut self) -> Option<Result<Endpoint, EndpointError>> {
        // if there's nothing in the set we just return
        if self.set.len() == 0 { return None; }

        // Iterate over all 
        while let Some(future) = self.set.pop() {
            // Check if the future is finished
            if !future.0.is_finished() {
                self.swap.push(future);
                continue;
            }

            // Put the swap queue back into the main one
            self.set.extend(self.swap.drain(..));

            // Poll the future and return it
            // This will panic instead of blocking
            // This is fine since we checked if the future was finished
            // If it does panic, it's a bug that should be fixed
            return Some(futures_lite::future::block_on(
                future.0.fallible()
            ).unwrap());
        }

        // Swap the sets
        // This is because swap now contains all of set
        mem::swap(&mut self.set, &mut self.swap);

        // Nothing done yet
        return None;
    }
}

impl Default for WaitingEndpoints {
    fn default() -> Self {
        Self::new()
    }
}

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