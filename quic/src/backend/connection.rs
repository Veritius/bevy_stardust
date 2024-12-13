use quinn_proto::ConnectionHandle;
use super::events::{C2EEvent, E2CEvent};

pub(crate) struct State {
    quinn: quinn_proto::Connection,

    endpoint_event_rx: async_channel::Receiver<E2CEvent>,
    connection_event_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,
}

pub(crate) enum Driver {}

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