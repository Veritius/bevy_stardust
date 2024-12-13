use quinn_proto::ConnectionHandle;
use super::events::{C2EEvent, E2CEvent};

pub(crate) struct Handle {
    close_signal_tx: async_channel::Sender<CloseSignal>,
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

    endpoint_event_rx: async_channel::Receiver<E2CEvent>,
    connection_event_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,
}

pub(super) enum Driver {}

impl Driver {
    pub fn run(
        state: State,
    ) {
        todo!()
    }
}


async fn driver(
    mut state: State,
) {
    todo!()
}