//! Common imports for server-side code.
//! 
//! `use bevy_stardust::server::prelude::*`

pub use crate::shared::prelude::*;
pub use crate::shared::channels::outgoing::SendTarget;
pub use crate::server::plugin::StardustServerPlugin;
pub use crate::server::connection::*;
pub use crate::server::settings::NetworkClientCap;
pub use crate::server::clients::Client;
pub use crate::server::receive::*;
pub use crate::server::send::*;