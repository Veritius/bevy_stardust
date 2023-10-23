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
    pub timeout: Duration,
    pub direction: PendingDirection,
}

// ============================================================================================
// You may notice that PendingOutgoingState is unnecessary since it's processed immediately
// by the reading system and never needs to be stored. However, I've decided to make it an enum
// for the sake of future proofing it, and saving me time in future.
// 
// If I decide to, say, add encryption, there'll need to be a step between the server
// acknowledging the peer and its acceptance, where cryptographic keys and stuff are exchanged.
// Besides, pending peers take up very little time overall, so this isn't losing much.
// ============================================================================================

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
    /// No response from the server.
    NoResponseYet,
    /// Accepted by the server. Port attached.
    Accepted(u16),
    /// Denied by the server.
    Denied, // TODO: Store reason
}

/// Current state of the pending incoming connection attempt.
/// Has no variants as it is currently useless.
#[derive(Debug)]
pub(super) enum PendingIncomingState {
    
}