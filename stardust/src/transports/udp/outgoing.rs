use std::{net::SocketAddr, time::{Instant, Duration}};
use bevy::prelude::*;
use crate::protocol::ProtocolId;
use super::{ports::PortBindings, TRANSPORT_IDENTIFIER, COMPAT_THIS_VERSION, established::Disconnected};

const ATTEMPT_DELAY: Duration = Duration::from_secs(5);

#[derive(Debug, Default, Resource)]
pub(super) struct OutgoingConnectionAttempts(pub Vec<OutgoingConnectionAttempt>);

impl OutgoingConnectionAttempts {
    pub fn get_attempt_from_address(&self, address: &SocketAddr) -> Option<(usize, &OutgoingConnectionAttempt)> {
        self.0.iter()
        .enumerate()
        .find(|p| &p.1.address == address)
    }
}

#[derive(Debug)]
pub(super) struct OutgoingConnectionAttempt {
    pub address: SocketAddr,
    pub local_idx: u16,
    pub started: Instant,
    pub last_sent: Option<Instant>,
    pub timeout: Duration,
}

pub(super) fn send_attempt_packets_system(
    ports: Res<PortBindings>,
    protocol: Res<ProtocolId>,
    mut attempts: ResMut<OutgoingConnectionAttempts>,
) {
    // Create a buffer for use in the send fn, this part of the message will not change.
    let mut message = [0u8; 23];
    message[0] = 0;
    message[1..9].copy_from_slice(&TRANSPORT_IDENTIFIER.to_be_bytes());
    message[9..13].copy_from_slice(&COMPAT_THIS_VERSION.to_be_bytes());
    message[13..21].copy_from_slice(&protocol.int().to_be_bytes());

    // Used for time comparisons
    let now = Instant::now();

    // Remove any attempts that have timed out
    let mut removals = vec![];
    for (idx, attempt) in attempts.0.iter().enumerate() {
        if now.duration_since(attempt.started) > attempt.timeout {
            removals.push(idx);
        }
    }
    for idx in removals.iter().rev() {
        attempts.0.remove(*idx);
    }
    attempts.0.shrink_to_fit();

    // Send packets for active attempts
    let socket = ports.iter().nth(0).unwrap().1;
    for attempt in attempts.0.iter_mut() {
        // Check time since a message was last sent
        if let Some(last_sent) = attempt.last_sent {
            if now.duration_since(last_sent) > ATTEMPT_DELAY {
                break
            }
        }

        // Update message
        message[21..23].copy_from_slice(&attempt.local_idx.to_be_bytes());

        // Send packet and update attempt
        socket.send_to(&message, attempt.address)
            .expect("Failed to send attempt message, this should not happen.");
        attempt.last_sent = Some(now);
    }
}

#[derive(Debug)]
pub(super) enum OutgoingAttemptResult {
    Accepted {
        rel_idx: u16,
        port: u16,
    },
    Rejected {
        reason: Disconnected,
    },
    BadResponse,
}