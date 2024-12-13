use bevy_stardust::prelude::ChannelMessage;
use quinn_proto::ConnectionHandle;
use super::{events::{C2EEvent, E2CEvent}, socket::DgramSend, taskpool::get_task_pool};

pub(crate) struct Handle {
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

pub(super) struct State {
    close_signal_rx: async_channel::Receiver<CloseSignal>,

    quinn: quinn_proto::Connection,
    handle: ConnectionHandle,

    endpoint_event_rx: async_channel::Receiver<E2CEvent>,
    connection_event_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,
    dgram_send_tx: async_channel::Sender<DgramSend>,

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
    todo!()
}