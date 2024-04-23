#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod entities;
mod events;
mod peer;
mod plugins;
mod scheduling;

pub mod change;
pub mod components;
pub mod diagnostics;
pub mod resources;
pub mod serialisation;

pub mod prelude {
    //! Common imports.

    use super::*;
    pub use change::*;
    pub use components::*;
    pub use entities::*;
    pub use events::*;
    pub use peer::*;
    pub use plugins::*;
    pub use resources::*;
    pub use scheduling::*;
    pub use serialisation::*;
}