use super::IncomingStreamData;
use super::IncomingStreamProcessingContext;
use super::ProcessingOutputAction;
use super::UnprocessedChunk::{self, *};

use bevy_stardust::channels::id::ChannelId;

pub(super) struct StardustPayloadsStream {
    pub channel_id: ChannelId,
}

impl IncomingStreamData for StardustPayloadsStream {
    fn process_chunk(
        &mut self,
        context: &IncomingStreamProcessingContext,
        chunk: UnprocessedChunk,
    ) -> ProcessingOutputAction {
        match chunk {
            Payload(mut reader) => {
                todo!()
            },
            Finished => todo!(),
            Reset(_) => todo!(),
        }
    }
}