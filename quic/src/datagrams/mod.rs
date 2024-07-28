mod header;
mod incoming;
mod outgoing;

use bytes::Bytes;
use crate::Connection;

impl Connection {
    /// Call when a datagram is received.
    pub fn recv_dgram(&mut self, dgram: Bytes) {
        todo!()
    }
}