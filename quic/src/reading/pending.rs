use bevy_stardust::channels::id::ChannelId;
use quinn_proto::Dir;

use super::IncomingStreamData;
use super::IncomingStreamProcessingContext;
use super::ProcessingOutputAction;
use super::UnprocessedChunk::{self, *};

use crate::streams::StreamErrorCode;
use crate::streams::StreamPurposeHeader;


pub(super) struct PendingStreamData;

impl IncomingStreamData for PendingStreamData {
    fn process_chunk(
        &mut self,
        context: &IncomingStreamProcessingContext,
        chunk: UnprocessedChunk,
    ) -> ProcessingOutputAction {
        match chunk {
            Payload(mut reader) => {
                // Get the purpose header that should be at the start of new streams
                let purpose_header = match reader.read_byte().ok() {
                    Some(val) => match StreamPurposeHeader::try_from(val).ok() {
                        Some(val) => val,
                        None => { return StreamErrorCode::InvalidOpeningHeader.into(); },
                    },
                    None => { return StreamErrorCode::InvalidOpeningHeader.into(); },
                };

                // Turn the purpose header into a valid stream state
                match purpose_header {
                    StreamPurposeHeader::ConnectionManagement => {
                        if context.stream_id.dir() != Dir::Uni { return StreamErrorCode::InvalidChannelDirection.into(); }

                        reader.commit_bytes(1);
                        // stream_data.data = IncomingStreamData::ConnectionManagement(ConnectionManagementStream);
                        return ProcessingOutputAction::DoNothing;
                    },

                    StreamPurposeHeader::StardustPayloads => {
                        if context.stream_id.dir() != Dir::Uni { return StreamErrorCode::InvalidChannelDirection.into(); }

                        let channel_id = match reader.read_bytes(4).ok() {
                            Some(val) => ChannelId::from(u32::from_be_bytes(TryInto::<[u8;4]>::try_into(val.as_slice_less_safe()).unwrap())),
                            None => { return StreamErrorCode::InvalidOpeningHeader.into(); },
                        };

                        reader.commit_bytes(5);
                        // stream_data.data = IncomingStreamData::StardustPayloads(StardustPayloadsStream { id: channel_id });
                        return ProcessingOutputAction::DoNothing;
                    },

                    StreamPurposeHeader::UctrlStream => {
                        todo!()
                    },
                }
            },

            Finished => todo!(),
            Reset(_) => todo!(),
        }
    }
}