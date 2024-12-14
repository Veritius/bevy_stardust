use std::{pin::pin, sync::Arc};
use bevy_stardust::prelude::ChannelMessage;
use futures_lite::StreamExt;
use quinn_proto::ConnectionHandle;
use super::{events::{C2EEvent, E2CEvent}, socket::DgramSend, taskpool::get_task_pool};

pub(crate) struct Handle {
    shared: Arc<Shared>,

    close_signal_tx: async_channel::Sender<CloseSignal>,

    message_recv_rx: crossbeam_channel::Receiver<ChannelMessage>,
    message_send_tx: async_channel::Sender<ChannelMessage>,
}

impl Drop for Handle {
    fn drop(&mut self) {
        // Signal the task to close (otherwise it wouldn't, as it's detached)
        let _ = self.close_signal_tx.send_blocking(CloseSignal {});
    }
}

struct CloseSignal {

}

struct Shared {

}

pub(super) struct State {
    shared: Arc<Shared>,

    close_signal_rx: async_channel::Receiver<CloseSignal>,

    quinn: quinn_proto::Connection,
    handle: ConnectionHandle,

    e2c_rx: async_channel::Receiver<E2CEvent>,
    c2e_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,
    dgram_tx: async_channel::Sender<DgramSend>,

    message_recv_tx: crossbeam_channel::Sender<ChannelMessage>,
    message_send_rx: async_channel::Receiver<ChannelMessage>,
}

pub(super) enum Driver {}

impl Driver {
    pub fn run(
        state: State,
    ) {
        get_task_pool()
            .spawn(driver(state))
            .detach();
    }
}

async fn driver(
    mut state: State,
) {
    enum Event {
        E2CEvent(E2CEvent),
        MessageSend(ChannelMessage),
        CloseSignal(CloseSignal),
    }

    let mut stream = pin!({
        let e2c_rx = state.e2c_rx.map(|v| Event::E2CEvent(v));
        let message_send_rx = state.message_send_rx.map(|v| Event::MessageSend(v));
        let close_signal_rx = state.close_signal_rx.map(|v| Event::CloseSignal(v));

        e2c_rx
            .or(message_send_rx)
            .or(close_signal_rx)
    });

    while let Some(event) = stream.next().await {
        match event {
            Event::E2CEvent(event) => todo!(),
            Event::MessageSend(channel_message) => todo!(),
            Event::CloseSignal(close_signal) => todo!(),
        }
    };
}