use std::collections::HashMap;
use quinn_proto::ConnectionHandle;
use super::events::{C2EEvent, E2CEvent};

pub(crate) struct State {
    quinn: quinn_proto::Endpoint,

    connections: HashMap<ConnectionHandle, Connection>,

    connection_event_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,
    connection_event_rx: async_channel::Receiver<(ConnectionHandle, C2EEvent)>,
}

struct Connection {
    event_tx: async_channel::Sender<E2CEvent>,
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