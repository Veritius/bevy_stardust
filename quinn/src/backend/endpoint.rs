use std::{collections::HashMap, future::Future, pin::Pin, sync::{Arc, Mutex}, task::{Context, Poll, Waker}, time::Instant};
use crossbeam_channel::{Receiver, Sender};
use quinn_proto::ConnectionHandle as ConnectionUid;
use super::{connection::ConnectionRef, socket::{AsyncUdpSocket, Receive, Transmit}};

#[derive(Clone)]
pub(crate) struct EndpointRef {
    ptr: Arc<Mutex<EndpointInner>>,
}

pub(super) struct EndpointInner {
    state: EndpointState,
    shared: Shared,
}

impl EndpointInner {
    pub fn new(
        config: EndpointConfig,
    ) -> Self {
        todo!()
    }
}

pub(crate) struct EndpointConfig {

}

enum EndpointState {
    Established(Established),
    Shutdown(Shutdown),
}

struct Established {
    quinn_proto: quinn_proto::Endpoint,
}

impl Established {
    fn handle_dgrams(
        &mut self,
        shared: &mut Shared,
    ) {
        let mut scratch = Vec::new();

        for dgram in shared.dgrams.try_iter() {
            if let Some(event) = self.quinn_proto.handle(
                Instant::now(),
                dgram.address,
                None,
                None,
                dgram.payload,
                &mut scratch,
            ) { match event {
                quinn_proto::DatagramEvent::ConnectionEvent(connection_handle, connection_event) => {
                    todo!()
                },

                quinn_proto::DatagramEvent::NewConnection(incoming) => {
                    // self.quinn_proto.accept(
                    //     incoming,
                    //     Instant::now(),
                    //     &mut scratch,
                    //     None,
                    // );

                    todo!()
                },

                quinn_proto::DatagramEvent::Response(transmit) => {
                    shared.socket.send(Transmit {
                        address: transmit.destination,
                        payload: &scratch[..],
                    });
                },
            } };
        }
    }
}

struct Shutdown {

}

struct ConnectionMap {
    map: HashMap<ConnectionUid, ConnectionHandle>,
}

struct ConnectionHandle {
    connection_ref: ConnectionRef,
    recv_events: Receiver<quinn_proto::EndpointEvent>,
    send_events: Sender<quinn_proto::ConnectionEvent>,
}

struct Shared {
    socket: AsyncUdpSocket,
    dgrams: Receiver<Receive>,

    waker: Option<Waker>,
}

struct EndpointDriver(EndpointRef);

impl Future for EndpointDriver {
    type Output = ();

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        let mut endpoint = self.0.ptr.lock().unwrap();

        let EndpointInner { state, shared } = &mut *endpoint;

        if shared.waker.is_none() {
            shared.waker = Some(cx.waker().clone());
        }

        match state {
            EndpointState::Established(established) => {
                established.handle_dgrams(shared);
            },

            EndpointState::Shutdown(shutdown) => todo!(),
        }

        todo!()
    }
}