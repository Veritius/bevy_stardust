use bevy_ecs::prelude::*;
use super::Endpoint;

/// Statistics related to an [`Endpoint`].
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature="reflect", derive(bevy_reflect::Reflect), reflect(from_reflect = false))]
pub struct EndpointStatistics {
    /// How many packets have been sent, in total.
    pub total_packets_sent: u64,

    /// How many packets have been received, in total.
    pub total_packets_received: u64,

    /// How many packets have been detected to be dropped, in total.
    pub total_packets_dropped: u64,

    /// How many bytes have been sent, in total.
    pub total_bytes_sent: u64,

    /// How many bytes have been received, in total.
    pub total_bytes_received: u64,

    /// How many packets have been sent, this tick.
    pub tick_packets_sent: u32,

    /// How many packets have been received, this tick.
    pub tick_packets_received: u32,

    /// How many bytes have been sent, this tick.
    pub tick_bytes_sent: u32,

    /// How many bytes have been received, this tick.
    pub tick_bytes_received: u32,
}

impl EndpointStatistics {
    pub(crate) fn track_send_packet(&mut self, bytes: usize) {
        self.total_packets_sent += 1;
        self.total_bytes_sent += bytes as u64;
        self.tick_packets_sent += 1;
        self.tick_bytes_sent += bytes as u32;
    }

    pub(crate) fn track_recv_packet(&mut self, bytes: usize) {
        self.total_packets_received += 1;
        self.total_bytes_received += bytes as u64;
        self.tick_packets_received += 1;
        self.tick_bytes_received += bytes as u32;
    }
}

pub(crate) fn reset_endpoint_statistics_system(
    mut endpoints: Query<&mut Endpoint>,
) {
    for mut endpoint in endpoints.iter_mut() {
        let statistics = &mut endpoint.statistics;
        statistics.tick_packets_sent = 0;
        statistics.tick_packets_received = 0;
        statistics.tick_bytes_sent = 0;
        statistics.tick_bytes_received = 0;
    }
}