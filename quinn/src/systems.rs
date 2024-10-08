use bevy_ecs::prelude::*;
use bevy_stardust_quic::{ConnectionEvent, StreamEvent};
use quinn_proto::Event as ApplicationEvent;
use crate::{access::*, connection::ConnectionInner, endpoint::EndpointInner};

pub(crate) fn event_exchange_system(
    mut parallel_iterator: ParEndpoints,
) {
    parallel_iterator.par_iter_all(|
        mut endpoint_access,
        mut connection_iterator,
    | {
        for mut connection_access in connection_iterator {

        }
    });
}

fn exchange_quinn_events(
    endpoint: &mut EndpointInner,
    connection: &mut ConnectionInner,
) {
    connection.quinn_handle_timeout();

    while let Some(event) = connection.quinn_poll_end() {
        if let Some(event) = endpoint.handle_event(connection.handle(), event) {
            connection.quinn_handle_event(event);
        }
    }
}

fn handle_qio_sm_events(
    endpoint: &mut EndpointInner,
    connection: &mut ConnectionInner,
) {
    connection.handle_qio_timeout();

    while let Some(event) = connection.qio_poll() {
        match event {
            ConnectionEvent::ReceivedMessage(channel_message) => todo!(),

            ConnectionEvent::StreamEvent(stream_event) => match stream_event {
                StreamEvent::Open { id } => todo!(),

                StreamEvent::Transmit { id, chunk } => todo!(),

                StreamEvent::SetPriority { id, priority } => todo!(),

                StreamEvent::Reset { id } => todo!(),

                StreamEvent::Finish { id } => todo!(),

                StreamEvent::Stop { id } => todo!(),
            },

            ConnectionEvent::TransmitDatagram(bytes) => todo!(),

            ConnectionEvent::Overheated => todo!(),
        }
    }
}

fn handle_application_events(
    endpoint: &mut EndpointInner,
    connection: &mut ConnectionInner,
) {
    while let Some(event) = connection.quinn_poll_app() {
        match event {
            ApplicationEvent::HandshakeDataReady => todo!(),
            ApplicationEvent::Connected => todo!(),
            ApplicationEvent::ConnectionLost { reason } => todo!(),
            ApplicationEvent::Stream(stream_event) => todo!(),
            ApplicationEvent::DatagramReceived => todo!(),
            ApplicationEvent::DatagramsUnblocked => todo!(),
        }
    }
}