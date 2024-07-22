//! Ordered byte streams.

mod config;
mod queue;

use crate::prelude::*;

pub use config::*;
pub use queue::*;

pub enum Streams {}

impl ChannelType for Streams {
    type Config = StreamConfiguration;
}