//! UDP datagrams contain multiple 'frames' in them, much like QUIC does.
//! This module only applies to post-handshake communication.

use bytes::Bytes;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DatagramFrameType {
    Padding,
    Ping,
    Ack,
    Manage,
    Payload,
}

impl TryFrom<u8> for DatagramFrameType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Padding),
            1 => Ok(Self::Ping),
            2 => Ok(Self::Ack),
            3 => Ok(Self::Manage),
            4 => Ok(Self::Payload),
            _ => Err(()),
        }
    }
}

impl From<DatagramFrameType> for u8 {
    fn from(value: DatagramFrameType) -> Self {
        match value {
            DatagramFrameType::Padding => 0,
            DatagramFrameType::Ping => 1,
            DatagramFrameType::Ack => 2,
            DatagramFrameType::Manage => 3,
            DatagramFrameType::Payload => 4,
        }
    }
}

pub(crate) struct DatagramFrame {
    pub frame_type: DatagramFrameType,
    pub frame_bytes: Bytes,
}