/// Configuration used in a [`Connection`](crate::Connection).
pub struct ConnectionConfig {
    /// The maximum amount of partially-received bytes buffered in a single stream.
    /// Exceeding this limit will lead to the channel being stopped with an error code.
    pub single_max_partial_recvs: usize,

    /// The maximum amount of partially-received bytes buffered across all streams.
    /// Exceeding this limit will lead to the immediate disconnection of the peer.
    pub overall_max_partial_recvs: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            single_max_partial_recvs: usize::MAX,
            overall_max_partial_recvs: 65535,
        }
    }
}