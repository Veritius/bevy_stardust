use super::frames::Frame;

/// Packs a queue of `Frame` objects into a single packet.
pub(crate) struct PacketBuilder {
    queue: Vec<Frame>,
}

impl Default for PacketBuilder {
    fn default() -> Self {
        Self {
            queue: Vec::with_capacity(32),
        }
    }
}

impl PacketBuilder {
    pub fn iter<'a>(&'a mut self) -> PacketBuilderIter<'a> {
        todo!()
    }

    pub(in crate::connection) fn push(&mut self, frame: Frame) {
        self.queue.push(frame);
    }
}

pub(crate) struct PacketBuilderIter<'a> {
    inner: &'a mut PacketBuilder,
}