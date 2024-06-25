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
pub use extension::*;