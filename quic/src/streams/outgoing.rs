use bevy_stardust::prelude::{ChannelId, Message};
use bytes::Bytes;
use crate::Connection;
use super::{header::StreamHeader, SendStreamId, StreamEvent};

const SEND_STREAM_ID_LIMIT: u64 = 2u64.pow(62) - 1;

pub(crate) struct OutgoingStreamsState {
    index: u64,
}

impl OutgoingStreamsState {
    pub fn new() -> Self {
        Self {
            index: 0,
        }
    }
}

impl Connection {
    pub(crate) fn send_message_on_stream<I>(&mut self, channel: ChannelId, message: Message) {
        todo!()
    }

    pub(crate) fn send_messages_on_stream<I>(&mut self, channel: ChannelId, iter: I)
    where
        I: Iterator<Item = Message>,
    {
        todo!()
    }

    #[inline]
    pub(crate) fn send_message_on_stream_and_close(&mut self, channel: ChannelId, message: Message) {
        self.send_transient_single(StreamHeader::Stardust { channel }, message.into())
    }

    #[inline]
    pub(crate) fn send_wrapped_dgram_single(&mut self, payload: Bytes) {
        self.send_transient_single(StreamHeader::WrappedDatagram, payload);
    }

    #[inline]
    pub(crate) fn send_wrapped_dgram_chunks<I>(&mut self, iter: I)
    where
        I: Iterator<Item = Bytes>,
    {
        self.send_transient_chunks(StreamHeader::WrappedDatagram, iter);
    }

    fn open_stream_inner(&mut self) -> SendStreamId {
        let index = self.outgoing_streams.index;
        assert!(index >= SEND_STREAM_ID_LIMIT, "Exceeded send ID limit");
        self.outgoing_streams.index += 1;
        let id = SendStreamId(index);
        self.stream_event(StreamEvent::Open { id });
        return id;
    }

    fn send_over_stream(&mut self, id: SendStreamId, data: Bytes) {
        self.stream_event(StreamEvent::Transmit { id, chunk: data })
    }

    fn finish_stream_inner(&mut self, id: SendStreamId) {
        self.stream_event(StreamEvent::Finish { id });
    }

    fn send_transient_single(&mut self, header: StreamHeader, payload: Bytes) {
        let id = self.open_stream_inner();
        self.send_over_stream(id, header.alloc());
        self.send_over_stream(id, payload);
        self.finish_stream_inner(id);
    }

    fn send_transient_chunks<I>(&mut self, header: StreamHeader, iter: I)
    where
        I: Iterator<Item = Bytes>,
    {
        let id = self.open_stream_inner();
        self.send_over_stream(id, header.alloc());

        for chunk in iter {
            self.send_over_stream(id, chunk);
        }

        self.finish_stream_inner(id);
    }
}