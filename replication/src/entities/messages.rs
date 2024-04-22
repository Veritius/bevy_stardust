use bevy::prelude::*;

/// Stardust channel for entity replication.
#[derive(Default, TypePath)]
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