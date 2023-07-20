pub mod args;
pub mod server;

use clap::Parser;
use log::info;
use crate::args::Args;


fn main() {
    env_logger::init();
    info!("bevy_stardust authentication server {}", env!("CARGO_PKG_VERSION"));

    let args = Args::parse();
}