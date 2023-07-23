use std::marker::PhantomData;
use bevy::prelude::Component;
use crate::channel::Channel;

/// Channel for component replication messages.
pub struct ComponentReplicationChannel<T: Component>(PhantomData<T>);
impl<T: Component> ComponentReplicationChannel<T> { fn new() -> Self { Self(PhantomData) }}
impl<T: Component> Channel for ComponentReplicationChannel<T> {}

impl<T: Component> std::fmt::Debug for ComponentReplicationChannel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentReplicationChannel").finish()
    }
}