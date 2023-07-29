//! Native UDP transport layer for clients.

use std::{sync::mpsc::{Receiver, self}, net::SocketAddr, thread::{JoinHandle, self}};

use bevy::prelude::*;
use crate::{shared::scheduling::{TransportReadPackets, TransportSendPackets}, client::connection::ConnectionRejectionReason};

/// A simple transport layer over native UDP sockets.
pub struct ClientUdpTransportPlugin;
impl Plugin for ClientUdpTransportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(TransportReadPackets, receive_packets_system);
        app.add_systems(TransportSendPackets, send_packets_system);
    }
}

fn receive_packets_system(
    
) {

}

fn send_packets_system(
    
) {

}

/// A running attempt to connect to a remote server.
#[derive(Debug)]
pub(super) struct ConnectionAttempt {
    receiver: Receiver<ConnectionAttemptUpdate>,
    remote: SocketAddr,
    handle: JoinHandle<()>,
}

/// The connection attempt has changed somehow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConnectionAttemptUpdate {
    /// The connection timed out.
    TimedOut,
    /// The server rejected the client.
    Rejected(ConnectionRejectionReason),
    /// The server wants the client to wait for authentication to occur.
    ServerWaitAuth,
    /// The server accepted the client.
    Accepted,
}

fn make_attempt(address: SocketAddr) -> ConnectionAttempt {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        let address = address.clone();
    });

    ConnectionAttempt {
        receiver,
        remote: address,
        handle,
    }
}