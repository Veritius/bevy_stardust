use bevy::ecs::world::Mut;
use bevy_stardust::prelude::*;
use super::{ordering::OrderedMessage, packets::{frames::RecvFrame, reader::PacketReadError}, OrderingManager};

pub(super) fn recv_stardust_frame(
    registry: &ChannelRegistryInner,
    orderings: &mut OrderingManager,
    messages: &mut Mut<NetworkMessages<Incoming>>,
    frame: RecvFrame,
) -> Result<(), PacketReadError>{
    // It's okay to unwrap here because the parser checked it
    let channel = ChannelId::try_from(frame.ident.unwrap()).unwrap();

    // Get the channel data from the registry based on the ident
    let channel_data = registry.channel_config(channel)
        .ok_or(PacketReadError::InvalidFrameIdent)?;

    // Quick check to see if the frame is ordered, if not, we can return early
    if channel_data.ordered == OrderingGuarantee::Unordered {
        messages.push(channel, frame.payload);
        return Ok(());
    }

    // If the frame is ordered we have to go through the ordering manager
    if frame.order.is_none() { return Err(PacketReadError::DefiedChannelExpectations); }
    let sequence = frame.order.unwrap();
    let ordering = orderings.get(channel_data);

    if let Some(r) = ordering.recv(OrderedMessage {
        sequence,
        payload: frame.payload,
    }) {
        // Push the message to the queue
        messages.push(channel, r.payload);

        // "Full order" channels have an extra step here
        // Since receiving a message can unlock previous messages
        // we have to try and drain the ordering manager for messages
        // TODO: Check if this should occur inside this if block or not
        if channel_data.ordered == OrderingGuarantee::Ordered {
            if let Some(drain) = ordering.drain_available() {
                for message in drain {
                    messages.push(channel, message.payload);
                }
            }
        }
    }

    return Ok(())
}