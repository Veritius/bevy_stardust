#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod change;
mod config;
mod entities;
mod messages;
mod plugins;
mod prediction;
mod resources;
mod rooms;
mod scheduling;
mod traits;

pub mod diagnostics;

pub mod prelude {
    //! Common imports.

    use super::*;
    pub use entities::*;
    pub use change::{NetChanges, NetChanged};
    pub use plugins::CoreReplicationPlugin;
    pub use resources::*;
    pub use rooms::*;
    pub use scheduling::*;
    pub use config::*;
    pub use traits::*;
}