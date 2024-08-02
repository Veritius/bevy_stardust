use bevy::{ecs::{query::{QueryData, QueryFilter, WorldQuery}, storage::TableRow}, prelude::*};

/// The lifestage of a connection.
/// 
/// This exists to model the average lifecycle of a connection, from an initial handshake to being disconnected.
/// An `Ord` implementation is provided, with variants being 'greater' if they're later in the model lifecycle.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect)]
#[reflect(Debug, Component, PartialEq)]
#[non_exhaustive]
pub enum PeerLifestage {
    /// Midway through a [handshake].
    /// Messages sent to peers in this stage will likely be ignored.
    /// 
    /// [handshake]: https://en.wikipedia.org/wiki/Handshake_(computing)
    Handshaking,

    /// Fully connected and communicating normally.
    Established,

    /// In the process of closing the connection.
    /// 
    /// This step may be skipped and peers will jump directly to the `Closed` stage from any other variant.
    Closing,

    /// The connection is closed.
    Closed,
}

/// A [`QueryFilter`] for entities in the [`Established`](PeerLifestage::Established) lifestage.
/// 
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_stardust::prelude::*;
/// #
/// fn my_system(query: Query<&Peer, Established>) {
///     for peer in &query {
///         println!("Hello, world!");
///     }
/// }
/// ```
#[derive(QueryData)]
pub struct Established<'w> {
    lifestage: Option<&'w PeerLifestage>,
}

impl<'w> QueryFilter for Established<'w> {
    const IS_ARCHETYPAL: bool = false;

    unsafe fn filter_fetch(
        fetch: &mut Self::Fetch<'_>,
        entity: Entity,
        table_row: TableRow,
    ) -> bool {
        Self::fetch(fetch, entity, table_row).lifestage
            .is_some_and(|e| *e == PeerLifestage::Established)
    }
}