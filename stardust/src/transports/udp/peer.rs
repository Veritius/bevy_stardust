use std::{net::SocketAddr, time::{Instant, Duration}};
use bevy::prelude::*;

/// A UDP peer that is fully connected.
#[derive(Debug, Component)]
pub(super) struct EstablishedUdpPeer {
    pub address: SocketAddr,
    /// The 'oopsies' meter. Increments when they do something weird and disconnects them if it gets too high.
    pub hiccups: u32,
}

/// A connection attempt to a remote peer.
#[derive(Debug, Component)]
pub(super) struct PendingUdpPeer {
    pub address: SocketAddr,
    pub started: Instant,
    pub timeout: Option<Duration>,
    pub direction: PendingDirection,
}

/// The direction and state of the connection attempt.
#[derive(Debug)]
pub(super) enum PendingDirection {
    /// Attempt from this peer to connect to a remote peer.
    Outgoing(PendingOutgoingState),
    /// Attempt from a remote peer to connect to this peer.
    Incoming(PendingIncomingState),
}

/// Current state of the pending outgoing connection attempt.
#[derive(Debug)]
pub(super) enum PendingOutgoingState {
    NoResponseYet,
}

/// Current state of the pending incoming connection attempt.
#[derive(Debug)]
pub(super) enum PendingIncomingState {
    
}