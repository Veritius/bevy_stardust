use bevy_ecs::prelude::*;
use crate::EndpointWeak;

/// Manages endpoints when used as a `Resource` in the world.
/// 
/// Deals with events, new connections, etc.
/// 
/// Requires [`QuicPlugin`](crate::QuicPlugin) to be added to be functional.
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
}