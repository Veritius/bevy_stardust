//! Collection and organisation of peers.
//! 
//! A [NetworkGroup] can contain [NetworkPeer] entities, or other `NetworkGroup`s.
//! When using network groups, no kind of hierarchy is accessible whatsoever.
//! Instead, peers can only be seen as either a member of a group, or not.
//! 
//! ## Hierarchy
//! Despite this, network groups do support a kind of hierarchy.
//! If a peer is a member of group A, and group A is added to group B,
//! the peer will then be a member of group A and B.
//! If the peer is then removed from group A, it will stop being a member of B.
//! But if a system tries to remove the peer from group B only, it will still be
//! considered a member of group B, since A is a member of B, and the peer is a member of A.
//! Trying to remove a member from a group it is not part of will put a warning in the console.
//! This same behavior applies to groups being members of groups.
//! 
//! Network group hierarchy requires you to be very careful when composing hierarchies of groups.
//! Additionally, changing large hierarchies can quickly become pretty computationally expensive.
//! 
//! Cyclic hierarchy patterns will inevitably crash the game, ie:
//! - making group A a member of group B, and making group B a member of group A
//! - a few more TODO WIP WRITE THIS
//! 
//! ## Management
//! You can manage network groups using the `Commands` or `NetworkGroups` systemparams.

use bevy::{prelude::*, ecs::system::{Command, SystemParam, SystemState}};
use crate::prelude::NetworkPeer;

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

/// Adds a [NetworkPeer] or [NetworkGroup] to a network group.
pub struct AddToNetworkGroup {
    /// The group to add the child to.
    pub group: Entity,
    /// The child to add to the group.
    pub child: Entity,
}

impl Command for AddToNetworkGroup {
    fn apply(self, world: &mut World) {
        let mut state: SystemState<(
            Query<&mut NetworkGroup>,
            Query<(), With<NetworkPeer>>,
        )> = SystemState::new(world);
        let (mut groups_query, peers_query) = state.get_mut(world);

        if !groups_query.contains(self.group) { return }
        if !(groups_query.contains(self.child) || peers_query.contains(self.child)) { return }

        let mut group = groups_query.get_mut(self.group).unwrap();
        if group.children.contains(&self.child) { return }
        group.children.push(self.child);
        group.children.sort_unstable();

        todo!()
    }
}

/// Removes `group` from `child`.
/// 
/// Logs a warning if `child` is not a direct child of `group`, since it will still be considered a member of `group`.
/// See [the module level documentation](self) for more on this behavior.
pub struct RemoveFromNetworkGroup {
    /// The group to remove the child from.
    pub group: Entity,
    /// The child to remove from the group.
    pub child: Entity,
}

impl Command for RemoveFromNetworkGroup {
    fn apply(self, world: &mut World) {
        let group = world.query::<&mut NetworkGroup>().get_mut(world, self.group);
        let mut group = if group.is_err() { return } else { group.unwrap() };
        match group.children.binary_search(&self.child) {
            Ok(idx) => {
                group.children.remove(idx);
                group.children.sort_unstable();
            },
            Err(_) => {
                warn!("Tried to remove {:?} from  network group {:?}, but it was not a direct child. The child may still be counted as a member of this group.", self.child, self.group);
                return
            },
        }

        todo!()
    }
}