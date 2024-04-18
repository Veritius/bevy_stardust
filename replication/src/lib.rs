#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod change;
mod entities;
mod events;
mod peer;
mod plugins;
mod prediction;
mod resources;
mod scheduling;

pub mod diagnostics;
pub mod rooms;
pub mod serialisation;

pub mod prelude {
    //! Common imports.

    use super::*;
    pub use change::{NetChanges, NetChanged};
    pub use entities::*;
    pub use events::*;
    pub use peer::*;
    pub use plugins::CoreReplicationPlugin;
    pub use resources::*;
    pub use rooms::*;
    pub use scheduling::*;
}