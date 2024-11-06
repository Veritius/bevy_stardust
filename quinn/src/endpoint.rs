use std::{future::Future, io, net::SocketAddr, sync::{Arc, Mutex, Weak}, time::Duration};
use bevy_ecs::prelude::*;
use bevy_tasks::{IoTaskPool, Task};
use bytes::BytesMut;
use crossbeam_channel::Receiver;
use mio::net::UdpSocket;
use crate::config::{ClientVerification, ServerAuthentication};

#[derive(Component)]
pub struct Endpoint {
    handle: EndpointHandle,
}

impl Endpoint {
    pub fn new(
        socket: SocketAddr,
        auth: ServerAuthentication,
        verify: ClientVerification,
    ) -> Endpoint {
        let handle = Arc::new(());

        let ptr = EndpointRef(Arc::new(EndpointInner {
            handle: handle.clone(),
            state: Mutex::new(EndpointState {
                quinn: todo!(),
            }),
        }));

        let task_pool = IoTaskPool::get();

        let task = EndpointTask::new(
            task_pool,
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
struct EndpointState {
    quinn: quinn_proto::Endpoint,
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
        task_pool: &IoTaskPool,
        config: EndpointTaskConfig,
    ) -> Self {
        let task = async move {

        };

        return Self(task_pool.spawn(task));
    }
}

/// Handle to an asynchronous task handling packet receiving.
struct IoRecvTask(Task<Option<io::Error>>);

impl IoRecvTask {
    fn new(
        task_pool: &IoTaskPool,
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

        let (tx,rx) = crossbeam_channel::unbounded();

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
            Self(task_pool.spawn(task)),
            socket_clone,
            rx,
        );
    }
}

pub(super) struct DgramRecv {
    pub address: SocketAddr,
    pub payload: BytesMut,
}

pub(super) struct DgramSend<'a> {
    pub address: SocketAddr,
    pub payload: &'a [u8],
}

fn would_block(err: &std::io::Error) -> bool {
    err.kind() == std::io::ErrorKind::WouldBlock
}