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
    /// Returns an object used to manage mutable parallel access to the query.
    pub fn par_access<'t>(&'t mut self) -> TransportIncomingWriterLockManager<'t, 'w, 's, Q> {
        TransportIncomingWriterLockManager {
            incoming: self
        }
    }
}

/// Manages mutable access to query entries in [TransportIncomingWriter] for parallelism purposes.
pub struct TransportIncomingWriterLockManager<'t, 'w, 's, Q>
where for <'a> Q: WorldQuery + 'a {
    incoming: &'t TransportIncomingWriter<'w, 's, Q>,
}

impl<'t, 'w, 's, Q> TransportIncomingWriterLockManager<'t, 'w, 's, Q>
where for <'a> Q: WorldQuery + 'a {
    /// Tries to lock mutable access to `peer`.
    pub fn try_lock<'a>(&self, peer: Entity) -> Option<()> {
        None
    }
}

/// Storage for network messages that have been received and directed to this peer.
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