#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod entities;
mod events;
mod peer;
mod plugins;
mod resources;
mod scheduling;

pub mod components;
pub mod diagnostics;
pub mod serialisation;

pub mod prelude {
    //! Common imports.

    use super::*;
    pub use entities::*;
    pub use events::*;
    pub use peer::*;
    pub use plugins::CoreReplicationPlugin;
    pub use resources::*;
    pub use scheduling::*;
    pub use serialisation::SerialisationFunctions;
}