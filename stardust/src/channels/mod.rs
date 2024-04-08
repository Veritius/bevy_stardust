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

pub use config::*;
pub use id::*;
pub use registry::*;
pub use extension::ChannelSetupAppExt;

use std::sync::Arc;
use bevy::prelude::*;

pub(super) fn channel_build(app: &mut App) {
    // Create setup channel registry
    app.insert_resource(registry::ChannelRegistryMut(Box::new(ChannelRegistryInner::new())));

}

pub(super) fn channel_finish(app: &mut App) {
    // Remove SetupChannelRegistry and put the inner into an Arc inside ChannelRegistry
    // This dramatically improves 
    let registry = app.world.remove_resource::<ChannelRegistryMut>().unwrap();
    app.insert_resource(ChannelRegistry(Arc::from(registry.0)));
}