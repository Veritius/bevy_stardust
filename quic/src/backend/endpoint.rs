use std::{collections::HashMap, pin::pin, sync::{Arc, Mutex}};
use futures_lite::StreamExt;
use quinn_proto::ConnectionHandle;
use crate::backend::socket::DgramRecv;
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
    state: Mutex<Lifestage>,
}

pub(super) struct State {
    shared: Arc<Shared>,
    lifestage: Lifestage,

    close_signal_rx: async_channel::Receiver<CloseSignal>,

    socket: Socket,

    quinn: quinn_proto::Endpoint,

    connections: HashMap<ConnectionHandle, Connection>,

    c2e_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,
    c2e_rx: async_channel::Receiver<(ConnectionHandle, C2EEvent)>,
}

impl State {
    fn update_lifestage(&mut self, lifestage: Lifestage) {
        self.lifestage = lifestage;
        *self.shared.state.lock().unwrap() = lifestage;
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Lifestage {
    Established,
    Closing,
    Closed
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
    enum Event {
        C2EEvent((ConnectionHandle, C2EEvent)),
        DgramRecv(DgramRecv),
        CloseSignal(CloseSignal),
    }

    let mut stream = pin!({
        let c2e_rx = state.c2e_rx.map(|v| Event::C2EEvent(v));
        let dgram_rx = state.socket.recv_rx.map(|v| Event::DgramRecv(v));
        let close_signal_rx = state.close_signal_rx.map(|v| Event::CloseSignal(v));

        c2e_rx
            .or(dgram_rx)
            .or(close_signal_rx)
    });

    while let Some(event) = stream.next().await {
        match event {
            Event::C2EEvent((handle, event)) => todo!(),
            Event::DgramRecv(dgram) => todo!(),
            Event::CloseSignal(signal) => todo!(),
        }
    }
}