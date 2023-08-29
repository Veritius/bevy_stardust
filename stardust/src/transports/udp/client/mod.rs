//! Native UDP transport layer for clients.

mod manager;
mod receiver;
mod sender;
mod attempt;

pub use manager::UdpClientManager;

use std::net::UdpSocket;
use bevy::prelude::{Plugin, App, Resource, OnTransition};
use crate::scheduling::*;
use self::{receiver::receive_packets_system, sender::send_packets_system, attempt::connection_attempt_system};

#[derive(Resource)]
struct RemoteServerUdpSocket(pub UdpSocket);