use std::{collections::HashMap, net::UdpSocket, pin::pin, sync::{Arc, Mutex}, time::Instant};
use async_io::Async;
use futures_lite::StreamExt;
use quinn_proto::ConnectionHandle;
use crate::{backend::{outgoing::RejectedData, socket::DgramRecv}, config::{EndpointConfig, ServerConfig}};
use super::{events::{C2EEvent, E2CEvent}, outgoing::{AcceptedData, OutgoingConnectionRequest}, socket::{Socket, SocketConfig}, taskpool::get_task_pool};

pub(crate) fn new(
    socket: Async<UdpSocket>,
    config: EndpointConfig,
    server: Option<ServerConfig>,
) -> Handle {
    let (close_signal_tx, close_signal_rx) = async_channel::unbounded();
    let (outgoing_request_tx, outgoing_request_rx) = async_channel::unbounded();
    let (c2e_tx, c2e_rx) = async_channel::unbounded();

    let shared = Arc::new(Shared {
        state: Mutex::new(Lifestage::Established),
    });

    let state = State {
        shared: shared.clone(),

        lifestage: Lifestage::Established,

        close_signal_rx,

        socket: Socket::new(
            socket,
            SocketConfig {
                recv_buf_size: config.recv_buf_size,
            },
        ),

        quinn: quinn_proto::Endpoint::new(
            config.quinn,
            server.clone().map(|v| v.quinn),
            true,
            None,
        ),

        connections: HashMap::new(),

        outgoing_request_rx,

        c2e_tx,
        c2e_rx,
    };

    // Spawn and detach the task to run it in the background
    get_task_pool().spawn(driver(state)).detach();

    return Handle {
        shared,

        close_signal_tx,
        outgoing_request_tx,
    };
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
        // TODO: See about avoiding cloning these receivers
        // This is done because creating the stream takes ownership of the channel,
        // so the state object can't be handed off to other functions as it's incomplete.
        let c2e_rx = state.c2e_rx.clone().map(|v| Event::C2EEvent(v));
        let dgram_rx = state.socket.recv_rx.clone().map(|v| Event::DgramRecv(v));
        let outgoing_request_rx = state.outgoing_request_rx.clone().map(|v| Event::OutgoingRequest(v));
        let close_signal_rx = state.close_signal_rx.clone().map(|v| Event::CloseSignal(v));

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
            Event::C2EEvent((handle, event)) => handle_c2e_event(&mut state, handle, event),
            Event::DgramRecv(dgram) => handle_dgram_recv(&mut state, dgram),
            Event::OutgoingRequest(request) => handle_outgoing_request(&mut state, request),
            Event::CloseSignal(signal) => handle_close_signal(&mut state, signal),
        }

        match state.lifestage {
            Lifestage::Established => todo!(),
            Lifestage::Closing => todo!(),
            Lifestage::Closed => todo!(),
        }
    }
}

fn handle_c2e_event(
    state: &mut State,
    handle: ConnectionHandle,
    event: C2EEvent,
) {
    todo!()
}

fn handle_dgram_recv(
    state: &mut State,
    dgram: DgramRecv,
) {
    todo!()
}

fn handle_outgoing_request(
    state: &mut State,
    request: OutgoingConnectionRequest,
) {
    // Check lifestage
    match state.lifestage {
        // In the established state, it's fine to accept requests, so we continue
        Lifestage::Established => {},

        // In the closing or closed states, we won't accept requests
        Lifestage::Closing | Lifestage::Closed => {
            // Inform the connection of the rejection
            request.split().1.reject(RejectedData {});
            return;
        },
    }

    let (params, request) = request.split();

    // Try to create the connection
    // Quinn handles most of this so we start here
    match state.quinn.connect(
        Instant::now(),
        params.config.quinn,
        params.remote_addr,
        &params.server_name,
    ) {
        Ok((handle, quinn)) => {
            // Channels for communication
            let (e2c_tx, e2c_rx) = async_channel::unbounded();

            // Inform task that it was accepted
            request.accept(AcceptedData {
                quinn,
                handle,

                e2c_rx,
                c2e_tx: state.c2e_tx.clone(),

                dgram_tx: state.socket.send_tx.clone(),
            });

            // Add connection to our set of values
            state.connections.insert(handle, Connection {
                e2c_tx,
            });
        },

        Err(_) => {
            // Inform task that it was rejected
            request.reject(RejectedData {});
        },
    }
}

fn handle_close_signal(
    state: &mut State,
    signal: CloseSignal,
) {
    todo!()
}