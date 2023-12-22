use crate::MAXIMUM_TRANSPORT_UNITS;

pub(super) struct Pipes {
    pipes: Vec<Pipe>,
}

impl Pipes {
    pub fn new(pipes: u8) -> Self {
        assert_ne!(pipes, u8::MIN);
        assert_ne!(pipes, u8::MAX);

        let mut pipes = Vec::with_capacity(pipes as usize);
        pipes.fill_with(|| Pipe {
            local: 0,
            remote: 0,
            bitfield: 0,
            unacked: [[0u8; MAXIMUM_TRANSPORT_UNITS]; 33],
        });

        Self {
            pipes,
        }
    }

    pub fn store(
        &mut self,
        pipe: u8,
        id: u16,
        ack: u16,
        bits: u32,
        message: &[u8],
    ) {

    }
}

pub(super) struct Pipe {
    local: u16,
    remote: u16,
    bitfield: u32,
    unacked: [[u8; MAXIMUM_TRANSPORT_UNITS]; 33],
}