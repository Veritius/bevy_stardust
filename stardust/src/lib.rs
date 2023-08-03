//! # bevy_stardust
//! A networking crate for the Bevy game engine.

pub mod client;
pub mod server;
pub mod shared;

pub mod replication;

mod impls;

/// Common things for writing client-side code.
/// 
/// Use like this: `use bevy_stardust::client_prelude::*;`
pub mod client_prelude {
    pub use crate::shared_prelude::*;
    pub use crate::client::plugin::StardustClientPlugin;
}

/// Common things for writing server-side code.
/// 
/// Use like this: `use bevy_stardust::server_prelude::*;`
pub mod server_prelude {
    pub use crate::shared_prelude::*;
    pub use crate::server::plugin::StardustServerPlugin;
}

/// Common things for writing shared code.
/// 
/// Use like this: `use bevy_stardust::shared_prelude::*;`
pub mod shared_prelude {
    pub use crate::shared::plugin::StardustSharedPlugin;
    pub use crate::shared::channels::extension::ChannelSetupAppExt;
    pub use crate::shared::channels::registry::ChannelRegistry;
}