use std::{collections::HashMap, sync::Arc};
use quinn_proto::ConnectionHandle;
use super::{events::{C2EEvent, E2CEvent}, socket::Socket, taskpool::get_task_pool};

pub(crate) struct Handle {
    shared: Arc<Shared>,

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

struct Shared {

}

pub(super) struct State {
    shared: Arc<Shared>,

    close_signal_rx: async_channel::Receiver<CloseSignal>,

    socket: Socket,

    quinn: quinn_proto::Endpoint,

    connections: HashMap<ConnectionHandle, Connection>,

    c2e_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,
    c2e_rx: async_channel::Receiver<(ConnectionHandle, C2EEvent)>,
}

struct Connection {
    e2c_tx: async_channel::Sender<E2CEvent>,
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