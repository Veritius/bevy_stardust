//! Replication of entity relations.

use std::{any::type_name, marker::PhantomData};
use aery::prelude::*;
use bevy::prelude::*;
use crate::entities::EntityReplicationPlugin;

/// Adds functionality for replicating entity relations from `aery`.
/// 
/// Relations between entities will only be replicated if both entities are replicated.
/// 
/// Requires [`EntityReplicationPlugin`] to be added beforehand.
pub struct RelationReplicationPlugin<R> {
    _p: PhantomData<R>,
}

impl<R> RelationReplicationPlugin<R>
where
    R: Relation
{
    /// Creates a new `RelationReplicationPlugin` for `R`.
    pub fn new() -> Self {
        Self {
            _p: PhantomData,
        }
    }
}

impl<R> Plugin for RelationReplicationPlugin<R>
where
    R: Relation,
{
    fn build(&self, app: &mut App) {
        assert!(app.is_plugin_added::<crate::entities::EntityReplicationPlugin>(),
            "{} requires {}, but it was not added", type_name::<Self>(), type_name::<EntityReplicationPlugin>());
    }
}