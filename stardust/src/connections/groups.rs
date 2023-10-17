//! Collection and organisation of peers.
//! 
//! A [NetworkGroup] can contain [NetworkPeer] entities, or other `NetworkGroup`s.
//! When using network groups, no kind of hierarchy is accessible whatsoever.
//! Instead, peers can only be seen as either a member of a group, or not.
//! 
//! Despite this, network groups do support a kind of hierarchy.
//! If a peer is a member of group A, and group A is added to group B,
//! the peer will then be a member of group A and B.
//! If the peer is then removed from group A, it will stop being a member of B.
//! But if a system tries to remove the peer from group B only, it will still be
//! considered a member of group B, since A is a member of B, and the peer is a member of A.
//! Trying to remove a member from a group it is not part of will put a warning in the console.
//! This same behavior applies to groups being members of groups.
//! 
//! Cyclic hierarchy patterns will crash the game, ie:
//! - making group A a member of group B, and making group B a member of group A
//! - a few more TODO WIP WRITE THIS

use bevy::{prelude::*, ecs::system::{Command, SystemParam}};

/// Methods for viewing network groups and peers they contain.
#[derive(SystemParam)]
pub struct NetworkGroups<'w, 's> {
    phantom_wip_remove_me: std::marker::PhantomData<(&'w (), &'s ())>
}

/// Added to `NetworkPeer` entities when they become a member of a group.
/// Lists any and all groups this peer is a part of, no matter how far in the hierarchy.
#[derive(Component)]
pub struct NetworkGroupMember(Vec<Entity>);

impl NetworkGroupMember {
    /// Returns all groups the entity is a member of.
    pub fn groups(&self) -> &[Entity] {
        &self.0
    }
}

/// A collection of peers and sub-groups.
/// See [the module level documentation](self) for more.
#[derive(Component, Reflect)]
pub struct NetworkGroup {
    ancestors: Vec<Entity>,
    children: Vec<Entity>,
}

impl NetworkGroup {
    /// Creates a new empty [NetworkGroup].
    pub fn new() -> Self {
        Self {
            ancestors: Vec::new(),
            children: Vec::new(),
        }
    }
}

/// Adds a peer or subgroup to a network group.
pub struct AddToNetworkGroup {
    /// The group to add the child to.
    pub group: Entity,
    /// The child to add to the group.
    pub child: Entity,
}

impl Command for AddToNetworkGroup {
    fn apply(self, world: &mut World) {
        todo!()
    }
}