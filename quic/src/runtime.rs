use std::{collections::VecDeque, io::ErrorKind, net::{SocketAddr, ToSocketAddrs}, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::JoinHandle};
use async_task::{Runnable, Task};
use bevy_ecs::system::Resource;
use bytes::BytesMut;
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use crossbeam_deque::Injector;

pub struct RuntimeBuilder {
    prefix: Box<str>,
    threads: usize,
}

impl RuntimeBuilder {
    pub fn new() -> RuntimeBuilder {
        RuntimeBuilder {
            prefix: "quic worker".into(),
            threads: 1,
        }
    }

    pub fn name(&mut self, name: impl Into<Box<str>>) {
        self.prefix = name.into();
    }

    pub fn threads(&mut self, count: usize) {
        self.threads = count;
    }
}

/// An owned handle to threads handling network traffic.
/// 
/// Can be used as a Bevy [`Resource`].
pub struct Runtime {
    prefix: Box<str>,
    workers: Vec<Worker>,
    state: Arc<State>,
}

impl Runtime {
    /// Returns the number of active worker threads.
    pub fn thread_count(&self) -> usize {
        self.workers.len()
    }

    /// Adds additional worker threads.
    pub fn add_workers(&mut self, amount: usize) {
        if amount == 0 { return } // do nothing

        let iter = (0..amount)
            .map(|_| Worker::new(
                self.prefix.clone(),
                self.state.clone(),
            ));

        self.workers.extend(iter);
    }

    /// Shuts down worker threads.
    pub fn remove_workers(&mut self, amount: usize) {
        todo!()
    }

    /// Returns the corresponding [`Handle`] for this [`Runtime`].
    pub fn handle(&self) -> Handle {
        Handle {
            state: self.state.clone(),
        }
    }
}

impl Resource for Runtime {}

/// A reference to the internal state of a [`Runtime`].
#[derive(Clone)]
pub struct Handle {
    state: Arc<State>,
}

/// A worker thread.
struct Worker {
    thread: JoinHandle<()>,
    shutdown: Arc<AtomicBool>,
}

impl Worker {
    fn new(
        name: impl Into<String>,
        state: Arc<State>,
    ) -> Worker {
        // Notification system for shutting down threads
        let shutdown = Arc::new(AtomicBool::new(false));

        // Start a new worker thread
        let thread_shutdown = shutdown.clone();
        let thread = std::thread::Builder::new()
            .name(name.into())
            .spawn(move || Worker::task(
                state,
                thread_shutdown,
            ))
            .unwrap();

        // We're done here
        return Worker {
            thread,
            shutdown,
        };
    }

    fn task(
        state: Arc<State>,
        shutdown: Arc<AtomicBool>,
    ) {
        // Thread-local queue of tasks needing completion
        let local = crossbeam_deque::Worker::new_lifo();

        loop {
            // Check if we've been signalled to shut down
            if shutdown.load(Ordering::Relaxed) {
                return; // Stop processing immediately
            }

            if local.is_empty() {
                // Fill up our local queue from the runtime's global queue
                state.tasks.steal_batch_with_limit(&local, 8);
            }

            // Run any tasks we have in our local queue
            if let Some(runnable) = local.pop() {
                runnable.run();
            }
        }
    }
}

/// Internal state for execution.
struct State {
    tasks: Injector<Runnable>,
}

struct Socket {
    socket: Arc<mio::net::UdpSocket>,
    dgram_rx: Receiver<DgramRecv>,
    dgram_tx: Sender<DgramSend>,
    thread: JoinHandle<Result<(), std::io::Error>>,
}

impl Socket {
    fn new(addr: impl ToSocketAddrs) -> Result<Self, std::io::Error> {
        // mio tokens as consts so they can be changed easily
        const TKN_READABLE: mio::Token = mio::Token(0);
        const TKN_WRITABLE: mio::Token = mio::Token(1);

        // Bind UDP socket and configure it
        let socket = std::net::UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;

        // Put udp socket in mio's wrapper type
        let mut socket = mio::net::UdpSocket::from_std(socket);

        // Channels for inter-thread communication
        let (dgram_recv_tx, dgram_recv_rx) = crossbeam_channel::unbounded::<DgramRecv>();
        let (dgram_send_tx, dgram_send_rx) = crossbeam_channel::unbounded::<DgramSend>();

        // Set up mio's polling system
        let mut mio_poll = mio::Poll::new()?;
        let mut mio_events = mio::Events::with_capacity(32);
        mio_poll.registry().register(&mut socket, TKN_READABLE, mio::Interest::READABLE);
        mio_poll.registry().register(&mut socket, TKN_WRITABLE, mio::Interest::WRITABLE);

        // Put socket in an arc since we're done mutating it
        let socket = Arc::new(socket);
        let thread_socket = socket.clone();

        // Start thread
        let thread = std::thread::spawn(move || {
            let socket = thread_socket;

            let mut blocked_sends: VecDeque<DgramSend> = VecDeque::with_capacity(1);

            loop {
                mio_poll.poll(&mut mio_events, None)?;

                'events: for event in mio_events.iter() {
                    match event.token() {
                        TKN_READABLE => {
                            // TODO: Allow configuring scratch size
                            let mut scratch = vec![0u8; 1472];

                            match socket.recv_from(&mut scratch[..]) {
                                Ok((length, address)) => match dgram_recv_tx.send(DgramRecv {
                                    origin: address,
                                    payload: {
                                        let mut buf = BytesMut::with_capacity(scratch.len());
                                        buf.copy_from_slice(&scratch[..length]);
                                        buf
                                    },
                                }) {
                                    Ok(_) => { continue },
                                    Err(_) => todo!(),
                                },

                                Err(_) => todo!(),
                            }
                        },

                        TKN_WRITABLE => {
                            while let Some(dgram) = blocked_sends.pop_front() {
                                match socket.send_to(&dgram.payload, dgram.target) {
                                    Ok(_) => {}, // Success

                                    // If this occurs we put it back into the queue
                                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                        blocked_sends.push_back(dgram);
                                        continue 'events;
                                    }

                                    // An actual I/O error occurred
                                    Err(e) => return Err(e),
                                }
                            }

                            loop { match dgram_send_rx.try_recv() {
                                Ok(dgram) => match socket.send_to(&dgram.payload, dgram.target) {
                                    Ok(_) => {}, // Success

                                    // If this occurs we queue it for attempted sending later on
                                    Err(e) if e.kind() == ErrorKind::WouldBlock => {
                                        blocked_sends.push_back(dgram);
                                        continue 'events;
                                    }
                                    
                                    // An actual I/O error occurred
                                    Err(e) => return Err(e),
                                },
    
                                Err(TryRecvError::Empty) => {
                                    continue 'events;
                                },
    
                                // If this occurs it means that the handle has been dropped
                                Err(TryRecvError::Disconnected) => { return Ok(()); }
                            } }

                        }

                        // Shouldn't happen
                        _ => unimplemented!(),
                    }
                }
            }
        });

        return Ok(Socket {
            socket,
            dgram_rx: dgram_recv_rx,
            dgram_tx: dgram_send_tx,
            thread,
        });
    }
}

pub(crate) struct DgramRecv {
    pub origin: SocketAddr,
    pub payload: BytesMut,
}

pub(crate) struct DgramSend {
    pub target: SocketAddr,
    pub payload: BytesMut,
}