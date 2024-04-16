use bevy::prelude::*;
use super::*;

pub(crate) fn update_hierarchy_system(
    mut commands: Commands,
    changed: Query<
        (Entity, &ReplicateHierarchy, Option<&Children>),
        (Changed<ReplicateHierarchy>, With<ReplicateEntity>),
    >,
    hierarchy: Query<(
        Entity,
        Option<&Parent>,
        Option<&Children>,
        Option<&ReplicateHierarchy>,
    )>,
) {
    todo!()
}