//! Channel definitions and message storage.
//! 
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_stardust::prelude::*;
//! 
//! #[derive(TypePath)]
//! struct MyChannel;
//! 
//! fn main() {
//!     let mut app = App::new();
//! 
//!     app.add_plugins((DefaultPlugins, StardustPlugin));
//! 
//!     app.add_channel::<MyChannel>(ChannelConfiguration {
//!         consistency: ChannelConsistency::ReliableUnordered,
//!         priority: 0,
//!     });
//! 
//!     app.run();
//! }
//! ```

mod config;
mod id;
mod registry;
mod extension;

pub use config::*;
pub use id::*;
pub use registry::*;
pub use extension::*;