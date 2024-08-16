# bevy_stardust_rooms
Organisational tools for collections of peers.

## Membership
A `Room` contains a set of 'members'. The members of a room are defined by its `Member` relations with other entities. An entity can have multiple `Member` relations.

A peer may be a member of a room by having a `Member` relation directly to it. This is called **direct membership** and is required to have a peer considered a member of any rooms.

When two rooms are linked by a `Member` relation, the target room includes all the members of the host room, but does not include itself. For example, if room A was a member of room B, room A would only contain direct members, but room B would contain direct members and all members of room A. This is called **indirect membership** and can cause a cycle (see [limitations](#limitations)).

### Limitations
Memberships must not create a cycle - for example, room A being a member of room B, which is a member of room A. If a relation is created that causes a cycle, the application will immediately hang. This is because of entity relations using `aery` which does not provide acyclic relations. See the [tracking issue](https://github.com/bevyengine/bevy/issues/3742) for entity relations in Bevy.