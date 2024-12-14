use std::{net::SocketAddr, pin::pin, sync::{Arc, Mutex}};
use bevy_stardust::prelude::ChannelMessage;
use futures_lite::StreamExt;
use quinn_proto::ConnectionHandle;
use crate::config::ClientConfig;
use super::{events::{C2EEvent, E2CEvent}, outgoing::{OutgoingConnectionReceiver, OutgoingRequestParams}, socket::DgramSend, taskpool::get_task_pool};

pub(crate) fn outgoing(
    endpoint: &super::endpoint::Handle,
    server_name: impl Into<Arc<str>>,
    remote_addr: SocketAddr,
    config: ClientConfig,
) -> Handle {
    // Construct request type used to communicate with the endpoint
    let (request, listener) = super::outgoing::new(OutgoingRequestParams {
        server_name: server_name.into(),
        remote_addr,
        config,
    });

    // Submit a request to the endpoint to create an outgoing connection
    endpoint.submit_outgoing_request(request);

    // Channels for communicating wtih the handle
    let (close_signal_tx, close_signal_rx) = async_channel::unbounded();
    let (message_recv_tx, message_recv_rx) = crossbeam_channel::unbounded();
    let (message_send_tx, message_send_rx) = async_channel::unbounded();

    let shared = Arc::new(Shared {
        state: Mutex::new(Lifestage::Connecting),
    });

    // Spawn and detach the task to run it in the background
    get_task_pool().spawn(outgoing_task(OutgoingTaskParams {
        shared: shared.clone(),
        listener,

        close_signal_rx,
        message_recv_tx,
        message_send_rx,
    })).detach();

    return Handle {
        shared,
        close_signal_tx,
        message_recv_rx,
        message_send_tx,
    };
}

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

impl Handle {
    pub fn state(&self) -> Lifestage {
        self.shared.state.lock().unwrap().clone()
    }

    pub fn send_close_signal(&self) {
        let _ = self.close_signal_tx.send_blocking(CloseSignal {});
    }

    pub fn queue_message_send(&self, message: ChannelMessage) {
        let _ = self.message_send_tx.send_blocking(message);
    }

    pub fn message_recv_iter(&self) -> impl Iterator<Item = ChannelMessage> + '_ {
        self.message_recv_rx.try_iter()
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

    quinn: quinn_proto::Connection,
    handle: ConnectionHandle,

    e2c_rx: async_channel::Receiver<E2CEvent>,
    c2e_tx: async_channel::Sender<(ConnectionHandle, C2EEvent)>,
    dgram_tx: async_channel::Sender<DgramSend>,

    message_recv_tx: crossbeam_channel::Sender<ChannelMessage>,
    message_send_rx: async_channel::Receiver<ChannelMessage>,
}

impl State {
    fn update_lifestage(&mut self, lifestage: Lifestage) {
        self.lifestage = lifestage;
        *self.shared.state.lock().unwrap() = lifestage;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Lifestage {
    Connecting,
    Connected,
    Closing,
    Closed
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

struct OutgoingTaskParams {
    shared: Arc<Shared>,
    listener: OutgoingConnectionReceiver,

    close_signal_rx: async_channel::Receiver<CloseSignal>,
    message_recv_tx: crossbeam_channel::Sender<ChannelMessage>,
    message_send_rx: async_channel::Receiver<ChannelMessage>,
}

async fn outgoing_task(
    params: OutgoingTaskParams,
) {
    // deconstruct
    let OutgoingTaskParams {
        shared,
        listener,

        close_signal_rx,
        message_recv_tx,
        message_send_rx,
    } = params;

    use super::outgoing::Response;

    // Wait for a response from the endpoint
    let response = match listener.rx.recv().await {
        Ok(response) => response,
        Err(_) => return,
    };

    // If we've been rejected, set some things and return
    let accepted = match response {
        Response::Accepted(data) => data,

        Response::Rejected(data) => {
            *shared.state.lock().unwrap() = Lifestage::Closed;
            return;
        },
    };

    // Create the state object for the driver
    let state = State {
        shared,

        lifestage: Lifestage::Connecting,

        close_signal_rx,

        quinn: accepted.quinn,
        handle: accepted.handle,

        e2c_rx: accepted.e2c_rx,
        c2e_tx: accepted.c2e_tx,

        dgram_tx: accepted.dgram_tx,

        message_recv_tx,
        message_send_rx,
    };

    // Run the connection to completion
    driver(state).await;
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

    loop {
        let event = match stream.next().await {
            Some(event) => event,
            None => todo!(),
        };

        match event {
            Event::E2CEvent(event) => todo!(),
            Event::MessageSend(message) => todo!(),
            Event::CloseSignal(signal) => todo!(),
        }

        match state.lifestage {
            Lifestage::Connecting => todo!(),
            Lifestage::Connected => todo!(),
            Lifestage::Closing => todo!(),
            Lifestage::Closed => todo!(),
        }
    }
}