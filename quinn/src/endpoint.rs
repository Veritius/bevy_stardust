use std::{future::Future, io, net::SocketAddr, sync::{Arc, Mutex, Weak}, time::Duration};
use async_task::Task;
use bevy_ecs::prelude::*;
use bytes::{Bytes, BytesMut};
use crossbeam_channel::{Receiver, Sender};
use mio::net::UdpSocket;
use crate::{commands::MakeEndpointInner, runtime::Runtime};

#[derive(Component)]
pub struct Endpoint {
    handle: EndpointHandle,
}

impl Endpoint {
    fn new(
        runtime: &Runtime,
        build: MakeEndpointInner,
    ) -> Endpoint {
        let handle = Arc::new(());

        let ptr = EndpointRef(Arc::new(EndpointInner {
            handle: handle.clone(),
            state: Mutex::new(EndpointState::Building(Building::new(
                runtime,
                build,
            ))),
        }));

        let task: EndpointTask = EndpointTask::new(
            runtime,
            EndpointTaskConfig {
                ptr: ptr.clone(),
            },
        );

        return Endpoint {
            handle: EndpointHandle {
                handle,
                task,
                ptr,
            }
        }
    }
}

struct EndpointHandle {
    handle: Arc<()>,
    task: EndpointTask,
    ptr: EndpointRef,
}

/// Endpoint data.
struct EndpointInner {
    handle: Arc<()>,

    state: Mutex<EndpointState>,
}

/// Mutable endpoint state.
enum EndpointState {
    Building(Building),
    Established(Established),
}

struct Building {
    task: Task<Result<Established, anyhow::Error>>,
}

impl Building {
    fn new(
        runtime: &Runtime,
        build: MakeEndpointInner,
    ) -> Self {
        match build {
            MakeEndpointInner::Preconfigured {
                socket,
                config,
                server,
            } => Building { task: runtime.spawn(async move {
                let endpoint = quinn_proto::Endpoint::new(
                    config,
                    server,
                    true,
                    None,
                );

                let mio_skt = mio::net::UdpSocket::from_std(
                    socket,
                );

                let (
                    recv_task,
                    socket_arc,
                    dgram_rx,
                ) = IoRecvTask::new(
                    runtime,
                    mio_skt,
                );

                let (
                    send_task,
                    dgram_tx,
                ) = IoSendTask::new(
                    runtime,
                    socket_arc,
                );

                Ok(Established {
                    quinn: endpoint,

                    dgram_recv_task: recv_task,
                    dgram_send_task: send_task,
                    
                    dgram_rx,
                    dgram_tx,
                })
            }) },
        }
    }
}

struct Established {
    quinn: quinn_proto::Endpoint,

    dgram_recv_task: IoRecvTask,
    dgram_send_task: IoSendTask,

    dgram_rx: Receiver<DgramRecv>,
    dgram_tx: Sender<DgramSend>,
}

/// A clonable, shared strong reference to an [`EndpointInner`].
#[derive(Clone)]
struct EndpointRef(Arc<EndpointInner>);

/// A clonable, shared weak reference to an [`EndpointInner`].
#[derive(Clone)]
struct EndpointRefWeak(Weak<EndpointInner>);

impl<'a> From<&'a EndpointRef> for EndpointRefWeak {
    fn from(value: &'a EndpointRef) -> Self {
        EndpointRefWeak(Arc::<EndpointInner>::downgrade(&value.0))
    }
}

/// Config used to build an [`EndpointTask`].
struct EndpointTaskConfig {
    ptr: EndpointRef
}

/// Handle to endpoint logic running asynchronously.
struct EndpointTask(Task<()>);

impl EndpointTask {
    fn new(
        runtime: &Runtime,
        config: EndpointTaskConfig,
    ) -> Self {
        let task = async move {

        };

        return Self(runtime.spawn(task));
    }
}

/// Handle to an asynchronous task handling packet receiving.
struct IoRecvTask(Task<Option<io::Error>>);

impl IoRecvTask {
    fn new(
        runtime: &Runtime,
        mut socket: UdpSocket,
    ) -> (
        IoRecvTask,
        Arc<UdpSocket>,
        Receiver<DgramRecv>
    ) {
        let mut poll = mio::Poll::new().unwrap();
        let mut events = mio::Events::with_capacity(128);
        
        poll.registry().register(
            &mut socket,
            mio::Token(0),
            mio::Interest::READABLE | mio::Interest::WRITABLE,
        ).unwrap();

        let socket = Arc::new(socket);
        let socket_clone = socket.clone();

        let (tx, rx) = crossbeam_channel::unbounded();

        let task = async move {
            let mut scratch = vec![0u8; 1472]; // TODO: Make configurable

            loop {
                poll.poll(&mut events, Some(Duration::ZERO)).unwrap();

                for _event in events.iter() {
                    loop {
                        match socket.recv_from(&mut scratch) {
                            Ok((length, address)) => {
                                let mut payload = BytesMut::with_capacity(length);
                                payload.extend_from_slice(&scratch[..length]);

                                // TODO: Handle errors
                                tx.send(DgramRecv { address, payload }).unwrap();
                            },

                            Err(ref err) if would_block(err) => break,
                            Err(err) => return Some(err),
                        }
                    }
                }
            }
        };

        return (
            Self(runtime.spawn(task)),
            socket_clone,
            rx,
        );
    }
}

/// Handle to an asynchronous task handling packet transmission.
struct IoSendTask(Task<Option<io::Error>>);

impl IoSendTask {
    fn new(
        runtime: &Runtime,
        socket: Arc<UdpSocket>,
    ) -> (
        IoSendTask,
        Sender<DgramSend>
    ) {
        let (tx, rx) = crossbeam_channel::unbounded::<DgramSend>();

        let task = async move {
            let mut iter = rx.iter();

            loop {
                let dgram = match iter.next() {
                    Some(dgram) => dgram,
                    None => return None,
                };

                match socket.send_to(
                    &dgram.payload,
                    dgram.address
                ) {
                    Ok(_) => continue,
                    Err(ref err) if would_block(err) => continue,

                    Err(err) => return Some(err),
                }
            }
        };

        return (
            Self(runtime.spawn(task)),
            tx,
        )
    }
}

pub(super) struct DgramRecv {
    pub address: SocketAddr,
    pub payload: BytesMut,
}

pub(super) struct DgramSend {
    pub address: SocketAddr,
    pub payload: Bytes,
}

fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}