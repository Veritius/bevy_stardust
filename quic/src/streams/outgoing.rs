use std::collections::BTreeMap;
use bevy_stardust::prelude::{ChannelId, Message};
use bytes::Bytes;
use crate::{datagrams::DatagramBuilder, Connection, ConnectionShared};
use super::{header::StreamHeader, SendStreamId, StreamEvent};

impl Connection {
    /// Call when a stream is stopped.
    pub fn stream_stopped(&mut self, stream: SendStreamId) {
        todo!()
    }

    pub(crate) fn send_dgram_over_stream(
        &mut self,
        datagram: DatagramBuilder,
    ) {
        
    }

    pub(crate) fn outgoing_streams_handle(&mut self) -> OutgoingStreamsHandle {
        OutgoingStreamsHandle::from(self)
    }
}

pub(crate) struct OutgoingStreams {
    unique_id_index: u64,
    channel_ids: BTreeMap<ChannelId, SendStreamId>,
}

impl OutgoingStreams {
    const SEND_STREAM_ID_LIMIT: u64 = 2u64.pow(62) - 1;

    pub fn new() -> Self {
        Self {
            unique_id_index: 0,
            channel_ids: BTreeMap::new(),
        }
    }
}

pub(crate) struct OutgoingStreamsHandle<'a> {
    pub data: &'a mut OutgoingStreams,
    pub shared: &'a mut ConnectionShared,
}

impl<'a> From<&'a mut Connection> for OutgoingStreamsHandle<'a> {
    fn from(value: &'a mut Connection) -> Self {
        Self {
            data: &mut value.outgoing_streams,
            shared: &mut value.shared,
        }
    }
}

impl OutgoingStreamsHandle<'_> {
    pub(crate) fn send_message_on_stream(&mut self, channel: ChannelId, message: Message) {
        let id = self.get_or_create_channel_stream(channel);

        self.send_over_stream(id, StreamHeader::Stardust { channel }.alloc());

        self.send_over_stream(id, message.into());
    }

    pub(crate) fn send_messages_on_stream<I>(&mut self, channel: ChannelId, iter: I)
    where
        I: Iterator<Item = Message>,
    {
        let id = self.get_or_create_channel_stream(channel);

        for message in iter {
            self.send_over_stream(id, message.into());
        }
    }

    #[inline]
    pub(crate) fn send_message_on_stream_and_close(&mut self, channel: ChannelId, message: Message) {
        self.send_transient_single(StreamHeader::Stardust { channel }, message.into())
    }

    #[inline]
    pub(crate) fn send_wrapped_dgram_single(&mut self, payload: Bytes) {
        self.send_transient_single(StreamHeader::Datagram, payload);
    }

    #[inline]
    pub(crate) fn send_wrapped_dgram_chunks<I>(&mut self, iter: I)
    where
        I: Iterator<Item = Bytes>,
    {
        self.send_transient_chunks(StreamHeader::Datagram, iter);
    }

    fn open_stream_inner(&mut self) -> SendStreamId {
        let index = self.data. unique_id_index;
        assert!(index < OutgoingStreams::SEND_STREAM_ID_LIMIT, "Exceeded send ID limit");
        self.data.unique_id_index += 1;
        let id = SendStreamId(index);
        self.shared.stream_event(StreamEvent::Open { id });
        return id;
    }

    fn send_over_stream(&mut self, id: SendStreamId, data: Bytes) {
        self.shared.stream_event(StreamEvent::Transmit { id, chunk: data })
    }

    fn finish_stream_inner(&mut self, id: SendStreamId) {
        self.shared.stream_event(StreamEvent::Finish { id });
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

    fn get_or_create_channel_stream(&mut self, channel: ChannelId) -> SendStreamId {
        match self.data.channel_ids.get(&channel) {
            Some(id) => return *id,
            None => {
                let id = self.open_stream_inner();
                self.data.channel_ids.insert(channel, id);
                return id;
            },
        }
    }
}