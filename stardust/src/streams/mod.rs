//! Ordered byte streams.

mod config;
mod queue;

use crate::prelude::*;

pub use config::*;
pub use queue::*;

/// A stream channel's unique ID.
pub type MessageCid = crate::channels::ChannelId<Streams>;

pub enum Streams {}

impl ChannelType for Streams {
    type Config = StreamConfiguration;
}