//! Common imports for client- or server-side network code.
//! 
//! `use bevy_stardust::shared::prelude::*`

pub use crate::shared::plugin::StardustSharedPlugin;
pub use crate::shared::channels::extension::ChannelSetupAppExt;
pub use crate::shared::channels::registry::ChannelRegistry;
pub use crate::shared::channels::outgoing::SendTarget;
pub use crate::shared::channels::config::*;
pub use crate::shared::scheduling::*;