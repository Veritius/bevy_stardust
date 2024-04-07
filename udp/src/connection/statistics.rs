use bevy::prelude::*;
use crate::Connection;

/// Statistics related to a [`Connection`].
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct ConnectionStatistics {
    /// How many packets this client has sent, in total.
    pub total_packets_sent: u64,

    /// How many packets this client has received, in total.
    pub total_packets_received: u64,

    /// How many packets this client has dropped, in total.
    pub total_packets_dropped: u64,

    /// How many messages this client has sent, in total.
    pub total_messages_sent: u64,

    /// How many messages this client has received, in total.
    pub total_messages_received: u64,

    /// How many packets this client has sent, this tick.
    pub tick_packets_sent: u32,

    /// How many packets this client has sent, this tick.
    pub tick_packets_received: u32,

    /// How many messages this client has sent, this tick.
    pub tick_messages_sent: u32,

    /// How many messages this client has sent, this tick.
    pub tick_messages_received: u32,
}

impl ConnectionStatistics {
    pub(crate) fn record_packet_send(&mut self, messages: usize) {
        self.total_packets_sent += 1;
        self.total_messages_sent += messages as u64;
        self.tick_packets_sent += 1;
        self.tick_messages_sent += messages as u32;
    }

    pub(crate) fn record_packet_recv(&mut self, messages: usize) {
        self.total_packets_received += 1;
        self.total_messages_received += messages as u64;
        self.tick_packets_received += 1;
        self.tick_messages_received += messages as u32;
    }
}

pub(crate) fn reset_connection_statistics_system(
    mut connections: Query<&mut Connection>,
) {
    for mut connection in connections.iter_mut() {
        let statistics = &mut connection.statistics;
        statistics.tick_packets_sent = 0;
        statistics.tick_packets_received = 0;
        statistics.tick_messages_sent = 0;
        statistics.tick_messages_received;
    }
}