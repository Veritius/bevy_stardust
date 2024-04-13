#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod change;
mod components;
mod entities;
mod messaging;
mod plugins;
mod resources;
mod rooms;
mod state;
mod traits;

pub mod diagnostics;

pub mod prelude {
    //! Common imports.

    use super::*;
    pub use change::{NetChanges, NetChanged};
    pub use components::*;
    pub use messaging::ReplicationChannelConfiguration;
    pub use plugins::CoreReplicationPlugin;
    pub use resources::*;
    pub use rooms::*;
    pub use traits::*;
    pub use state::*;
}