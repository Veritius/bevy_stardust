mod pipes;

pub(crate) use pipes::pipe_for_channel;

use std::ops::IndexMut;
use self::pipes::Pipe;

#[derive(Debug, Clone, Copy)]
pub struct ReliableHeader {
    pub seq: u16,
    pub ack: u16,
    pub bits: u32,
}

pub(crate) struct ReliabilityData {
    pipes: Vec<Pipe>,
}

impl ReliabilityData {
    pub fn new(pipes: u8) -> Self {
        assert_ne!(pipes, u8::MIN);
        assert_ne!(pipes, u8::MAX);

        let mut pipes = Vec::with_capacity(pipes as usize);
        pipes.fill_with(|| Pipe::new());

        Self {
            pipes,
        }
    }

    pub fn send(
        &mut self,
        pipe: u8,
        data: &[u8],
    ) -> ReliableHeader {
        todo!()
    }

    pub fn receive(
        &mut self,
        pipe: u8,
        header: &ReliableHeader,
        message: &[u8]
    ) {
        let mut pipe = self.pipes.index_mut(pipe as usize);
        todo!()
    }
}