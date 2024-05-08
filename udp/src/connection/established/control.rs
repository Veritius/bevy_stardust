use bytes::Bytes;
use crate::{connection::packets::builder::PacketBuilder, varint::VarInt};

/// Connection management state for an established connection.
pub(super) struct Controller {
    error_counter: u16,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            error_counter: 0,
        }
    }
}

impl Controller {
    pub fn track_error(
        &mut self,
        severity: ErrorSeverity,
    ) {
        let offset = match severity {
            ErrorSeverity::Minor => 10,
            ErrorSeverity::Major => 50,
            ErrorSeverity::Critical => 200,
            ErrorSeverity::Custom(v) => v,
        };

        self.error_counter = self.error_counter.saturating_add(offset);
    }

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