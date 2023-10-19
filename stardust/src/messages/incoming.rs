//! Systemparams for transport layers to read and store messages for game systems.

use std::collections::BTreeMap;
use bevy::{prelude::*, ecs::{system::SystemParam, query::WorldQuery}};
use crate::prelude::{ChannelId, OctetString};

// Welcome to lifetime hell.

/// Allows transport layers to store incoming messages on entities for game systems to read.
#[derive(SystemParam)]
pub struct TransportIncomingWriter<'w, 's, Q>
where for <'a> Q: WorldQuery + 'a {
    commands: Commands<'w, 's>,
    peers: Query<'w, 's, (&'static mut NetworkMessageStorage, Q)>,
}

impl<'w, 's, Q> TransportIncomingWriter<'w, 's, Q>
where for <'a> Q: WorldQuery + 'a {

}

/// Manages mutable access to [TransportIncomingWriter] for parallelism purposes.
pub struct TransportIncomingWriterLockManager<'w, 's, Q>
where for <'a> Q: WorldQuery + 'a {
    incoming: &'w TransportIncomingWriter<'w, 's, Q>,
}

impl<'w, 's, Q> TransportIncomingWriterLockManager<'w, 's, Q>
where for <'a> Q: WorldQuery + 'a {
    
}

/// Storage for network messages that have been directed to this entity.
// TODO: Finish TransportIncomingWriter and make this pub(super)
#[derive(Component)]
pub struct NetworkMessageStorage(BTreeMap<ChannelId, Vec<OctetString>>);

// Tests that Rust's type system and Bevy's code don't explode with this systemparam.
#[test]
fn transport_incoming_writer_typechecker() {
    use bevy::ecs::system::SystemState;
    let mut world = World::new();
    let mut state: SystemState<TransportIncomingWriter<()>> = SystemState::new(&mut world);
    state.get_mut(&mut world);
}