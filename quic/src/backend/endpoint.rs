use std::{collections::HashMap, net::UdpSocket, pin::pin, sync::{Arc, Mutex}};
use futures_lite::StreamExt;
use quinn_proto::ConnectionHandle;
use crate::{backend::socket::DgramRecv, config::{EndpointConfig, ServerConfig}};
use super::{events::{C2EEvent, E2CEvent}, outgoing::OutgoingConnectionRequest, socket::Socket, taskpool::get_task_pool};

pub(crate) fn new(
    socket: UdpSocket,
    config: EndpointConfig,
    server: Option<ServerConfig>,
) -> Handle {
    todo!()
}

pub(crate) struct Handle {
    shared: Arc<Shared>,

    close_signal_tx: async_channel::Sender<CloseSignal>,
    outgoing_request_tx: async_channel::Sender<OutgoingConnectionRequest>,
}

impl Drop for Handle {
    fn drop(&mut self) {
        // Signal the task to close (otherwise it wouldn't, as it's detached)
        let _ = self.close_signal_tx.send_blocking(CloseSignal {});
    }
}

impl Handle {
    pub fn state(&self) -> Lifestage {
        self.shared.state.lock().unwrap().clone()
    }

    pub fn send_close_signal(&self) {
        let _ = self.close_signal_tx.send_blocking(CloseSignal {});
    }

    pub(super) fn submit_outgoing_request(&self, request: OutgoingConnectionRequest) {
        let _ = self.outgoing_request_tx.send_blocking(request);
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
    outgoing_request_rx: async_channel::Receiver<OutgoingConnectionRequest>,

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
pub(crate) enum Lifestage {
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
        OutgoingRequest(OutgoingConnectionRequest),
        CloseSignal(CloseSignal),
    }

    let mut stream = pin!({
        let c2e_rx = state.c2e_rx.map(|v| Event::C2EEvent(v));
        let dgram_rx = state.socket.recv_rx.map(|v| Event::DgramRecv(v));
        let outgoing_request_rx = state.outgoing_request_rx.map(|v| Event::OutgoingRequest(v));
        let close_signal_rx = state.close_signal_rx.map(|v| Event::CloseSignal(v));

        c2e_rx
            .or(dgram_rx)
            .or(outgoing_request_rx)
            .or(close_signal_rx)
    });

    loop {
        let event = match stream.next().await {
            Some(event) => event,
            None => todo!(),
        };

        match event {
            Event::C2EEvent((handle, event)) => todo!(),
            Event::DgramRecv(dgram) => todo!(),
            Event::OutgoingRequest(request) => todo!(),
            Event::CloseSignal(signal) => todo!(),
        }

        match state.lifestage {
            Lifestage::Established => todo!(),
            Lifestage::Closing => todo!(),
            Lifestage::Closed => todo!(),
        }
    }
}