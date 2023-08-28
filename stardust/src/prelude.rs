//! Common imports for using Stardust.
//! 
//! You can use the modules `client` and `server` for client- and server-related preludes as well.

pub use crate::setup::*;
pub use crate::state::*;
pub use crate::channels::extension::ChannelSetupAppExt;
pub use crate::channels::registry::ChannelRegistry;
pub use crate::channels::id::{Channel, ChannelId};
pub use crate::channels::config::*;

/// Client-side prelude.
pub mod client {
    pub use crate::client;
    pub use client::connection::RemoteConnectionStatus;
    pub use client::peers::Server;
    pub use client::receive::ChannelReader;
    pub use client::send::ChannelWriter;
}

/// Server-side prelude.
pub mod server {
    pub use crate::server;
    pub use server::settings::*;
    pub use server::connection::*;
    pub use server::clients::*;
    pub use server::receive::ChannelReader;
    pub use server::send::ChannelWriter;
}