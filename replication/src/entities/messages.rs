use std::marker::PhantomData;
use bevy::prelude::*;

/// Stardust channel for entity replication.
#[derive(Default)]
pub(super) struct EntityReplicationChannel;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum EntityMessageHeader {
    Spawn,
    Despawn,
}

impl TryFrom<u8> for EntityMessageHeader {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        return Ok(match value {
            0 => Self::Spawn,
            1 => Self::Despawn,
            _ => { return Err(()); }
        });
    }
}

impl From<EntityMessageHeader> for u8 {
    fn from(value: EntityMessageHeader) -> Self {
        match value {
            EntityMessageHeader::Spawn => 0,
            EntityMessageHeader::Despawn => 1,
        }
    }
}

/// Stardust channel for component replication for type `T`.
#[derive(Default)]
pub(crate) struct ComponentReplicationChannel<T: Component>(PhantomData<T>);

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ComponentMessageHeader {
    Insert,
    Update,
    Remove,
}

impl TryFrom<u8> for ComponentMessageHeader {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        return Ok(match value {
            0 => Self::Insert,
            1 => Self::Update,
            2 => Self::Remove,
            _ => { return Err(()); }
        });
    }
}

impl From<ComponentMessageHeader> for u8 {
    fn from(value: ComponentMessageHeader) -> Self {
        match value {
            ComponentMessageHeader::Insert => 0,
            ComponentMessageHeader::Update => 1,
            ComponentMessageHeader::Remove => 2,
        }
    }
}
