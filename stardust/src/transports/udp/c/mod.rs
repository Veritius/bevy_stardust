//! Native UDP transport layer for clients.

pub mod manager;

mod receiver;
mod sender;
mod attempt;

use std::net::UdpSocket;
use bevy::prelude::{Plugin, App, Resource, OnTransition};
use crate::scheduling::*;
use self::{receiver::receive_packets_system, sender::send_packets_system, attempt::connection_attempt_system};

#[derive(Resource)]
struct RemoteServerUdpSocket(pub UdpSocket);