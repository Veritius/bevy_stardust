mod pipes;

use std::ops::IndexMut;
use self::pipes::Pipe;

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

    pub fn store(
        &mut self,
        pipe: u8,
        seq: u16,
        ack: u16,
        bits: u32,
        message: &[u8]
    ) {
        let mut pipe = self.pipes.index_mut(pipe as usize);
        todo!()
    }
}

#[inline]
pub(crate) const fn pipe_for_channel(pipes: u8, channels: u32, channel: u32) -> u8 {
    let channels_per_pipe = channels / pipes as u32;
    (channel / channels_per_pipe) as u8
}