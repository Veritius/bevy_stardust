use std::time::Instant;
use bytes::Bytes;
use crate::{connection::packets::builder::PacketBuilder, varint::VarInt};

/// Connection management state for an established connection.
pub(super) struct Controller {
    error_counter: u16,

    close_state: Option<CloseState>,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            error_counter: 0,

            close_state: None,
        }
    }
}

impl Controller {
    pub fn track_error(&mut self, severity: ErrorSeverity) {
        let offset = match severity {
            ErrorSeverity::Minor => 10,
            ErrorSeverity::Major => 50,
            ErrorSeverity::Critical => 200,
            ErrorSeverity::Custom(v) => v,
        };

        self.error_counter = self.error_counter.saturating_add(offset);
    }

    const CTRL_FRAME_KEEP_ALIVE: VarInt = VarInt::from_u32(0);
    const CTRL_FRAME_CLOSE_CONN: VarInt = VarInt::from_u32(1);

    pub fn recv_control_frame(&mut self, ident: VarInt, bytes: Bytes) {
        todo!()
    }

    pub fn send_control_frame(&mut self, builder: &mut PacketBuilder) {
        todo!()
    }
}

pub(crate) enum ErrorSeverity {
    Minor,
    Major,
    Critical,
    Custom(u16),
}

struct CloseState {
    reason: Option<Bytes>,
    time: Instant,
}