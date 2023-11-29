//! Channel definitions and message storage.
//! 
//! You can add a channel when setting up the `App`.
//! ```
//! use bevy::prelude::*;
//! use bevy_stardust::prelude::*;
//! 
//! #[derive(Debug, Reflect)]
//! struct MyChannel;
//! 
//! fn main() {
//!     let mut app = App::new();
//! 
//!     app.register_channel::<MyChannel>(ChannelConfiguration {
//!         reliable: ChannelReliability::Reliable,
//!         ordering: ChannelOrdering::Ordered,
//!         fragmentation: ChannelFragmentation::Disabled,
//!         compression: ChannelCompression::Disabled,
//!         validation: MessageValidation::Disabled,
//!     });
//! }
//! ```

pub mod config;
pub mod id;
pub mod registry;

pub(crate) mod extension;
pub(crate) mod incoming;
pub(crate) mod outgoing;

pub(super) fn channels(app: &mut bevy::prelude::App) {
    use bevy::prelude::*;
    use crate::scheduling::*;

    // Channel registry
    app.insert_resource(registry::ChannelRegistry::new());

    // Clearing systems
    app.add_systems(PostUpdate, (incoming::clear_incoming, outgoing::clear_outgoing)
        .after(NetworkWrite::Send).in_set(NetworkWrite::Clear));
}

static CHANNEL_ENTITY_DELETED_MESSAGE: &'static str = "A channel entity was deleted or somehow stopped being accessible to a query. This should not happen!";