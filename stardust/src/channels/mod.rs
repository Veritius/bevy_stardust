//! Channel definitions and message storage.
//! 
//! You can add a channel when setting up the `App`.
//! ```ignore
//! #[derive(Reflect)] // Only necessary with the reflect feature
//! struct MyChannel;
//! 
//! fn main() {
//!     let mut app = App::new();
//! 
//!     app.add_plugins((DefaultPlugins, StardustPlugin));
//! 
//!     app.add_channel::<MyChannel>(ChannelConfiguration {
//!         reliable: ReliabilityGuarantee::Unreliable,
//!         ordered: OrderingGuarantee::Unordered,
//!         fragmented: false,
//!         string_size: 0..=16,
//!     });
//! }
//! ```

mod config;
mod id;
mod registry;
mod extension;
mod incoming;
mod outgoing;

pub use config::*;
pub use id::*;
pub use registry::*;
pub use extension::ChannelSetupAppExt;
pub use incoming::{NetworkIncomingWriter, NetworkReader};
pub use outgoing::{NetworkOutgoingReader, NetworkWriter};

pub(super) fn channels(app: &mut bevy_app::App) {
    use bevy_app::prelude::*;
    use bevy_ecs::prelude::*;
    use crate::scheduling::*;

    // Channel registry
    app.insert_resource(registry::ChannelRegistry::new());

    // Clearing systems
    app.add_systems(PostUpdate, (incoming::clear_incoming, outgoing::clear_outgoing)
        .after(NetworkWrite::Send).in_set(NetworkWrite::Clear));
}

static CHANNEL_ENTITY_DELETED_MESSAGE: &'static str = "A channel entity was deleted or somehow stopped being accessible to a query. This should not happen!";