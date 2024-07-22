use bytes::Bytes;
use super::streams::StreamId;

pub enum StreamEvent {
    StreamOpened {
        id: StreamId,
    },

    StreamFinished {
        stream_id: StreamId,
    },

    StreamReset {
        stream_id: StreamId,
        error_code: (),
    },

    ChunkReceived {
        stream_id: StreamId,
        payload: Bytes,
    },
}

pub enum DatagramEvent {
    DatagramReceived {
        payload: Bytes,
    }
}