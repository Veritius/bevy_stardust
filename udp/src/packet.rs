use std::collections::VecDeque;
use bytes::Bytes;

pub(crate) struct PacketQueue {
    incoming: VecDeque<IncomingPacket>,
    outgoing: VecDeque<OutgoingPacket>,
}

impl PacketQueue {
    pub fn push_incoming(&mut self, item: IncomingPacket) {
        self.incoming.push_back(item);
    }

    pub fn push_outgoing(&mut self, item: OutgoingPacket) {
        self.outgoing.push_back(item);
    }

    pub fn pop_incoming(&mut self) -> Option<IncomingPacket> {
        self.incoming.pop_front()
    }

    pub fn pop_outgoing(&mut self) -> Option<OutgoingPacket> {
        self.outgoing.pop_front()
    }

    pub fn incoming(&self) -> &VecDeque<IncomingPacket> {
        &self.incoming
    }

    pub fn outgoing(&self) -> &VecDeque<OutgoingPacket> {
        &self.outgoing
    }
}

impl PacketQueue {
    pub fn new(
        incoming_capacity: usize,
        outgoing_capacity: usize
    ) -> Self {
        Self {
            incoming: VecDeque::with_capacity(incoming_capacity),
            outgoing: VecDeque::with_capacity(outgoing_capacity),
        }
    }
}

pub(crate) struct IncomingPacket {
    pub payload: Bytes,
}

pub(crate) struct OutgoingPacket {
    pub payload: Bytes,
    pub messages: u32,
}