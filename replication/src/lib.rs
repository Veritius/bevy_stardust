#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod components;
mod plugin;
mod resources;
mod rooms;
mod traits;

pub use components::*;
pub use plugin::ReplicationPlugin;
pub use resources::*;
pub use rooms::*;
pub use traits::Replicable;