use super::IncomingStreamData;
use super::IncomingStreamProcessingContext;
use super::ProcessingOutputAction;
use super::UnprocessedChunk::{self, *};

pub(super) struct UctrlStream;

impl IncomingStreamData for UctrlStream {
    fn process_chunk(
        &mut self,
        context: &IncomingStreamProcessingContext,
        chunk: UnprocessedChunk,
    ) -> ProcessingOutputAction {
        match chunk {
            Payload(_) => todo!(),
            Finished => todo!(),
            Reset(_) => todo!(),
        }
    }
}