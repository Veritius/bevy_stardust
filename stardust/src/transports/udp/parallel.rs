//! Types for assisting in parallelism within the transport layer.

use bevy::{prelude::*, ecs::system::{CommandQueue, Command}};

/// A [CommandQueue] that is itself a `Command`. Used by taskpools for deferred World mutations.
pub(super) struct DeferredCommandQueue(pub CommandQueue);

impl Command for DeferredCommandQueue {
    fn apply(mut self, world: &mut World) {
        self.0.apply(world)
    }
}