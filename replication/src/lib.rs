#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod change;
mod entities;
mod events;
mod peer;
mod plugins;
mod resources;
mod scheduling;

pub mod diagnostics;
pub mod serialisation;
pub mod visibility;

pub mod prelude {
    //! Common imports.

    use super::*;
    pub use change::NetChanges;
    pub use entities::*;
    pub use events::*;
    pub use peer::*;
    pub use plugins::CoreReplicationPlugin;
    pub use resources::*;
    pub use scheduling::*;
    pub use serialisation::SerialisationFunctions;
}