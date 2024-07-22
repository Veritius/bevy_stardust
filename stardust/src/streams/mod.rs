//! Ordered byte streams.

mod chunk;
mod config;
mod queue;

use crate::prelude::*;

pub use config::*;
pub use queue::*;

/// A stream channel's unique ID.
pub type StreamCid = crate::channels::ChannelId<Streams>;

pub enum Streams {}

impl ChannelType for Streams {
    type Config = StreamConfiguration;
}