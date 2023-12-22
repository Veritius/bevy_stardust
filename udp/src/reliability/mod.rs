mod pipes;

use self::pipes::Pipes;

pub(crate) struct ReliabilityData {
    pipes: Pipes,
}

impl ReliabilityData {
    pub fn new(pipes: u8) -> Self {
        Self {
            pipes: Pipes::new(pipes),
        }
    }
}

#[inline]
pub(crate) const fn pipe_for_channel(pipes: u8, channels: u32, channel: u32) -> u8 {
    let channels_per_pipe = channels / pipes as u32;
    (channel / channels_per_pipe) as u8
}