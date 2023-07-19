use std::{sync::{mpsc::{self, Receiver}, Arc, Mutex}, thread, collections::HashMap, net::{SocketAddr, Shutdown}, io};
use bevy::prelude::{Resource, error, debug};
use bevy_stardust_shared::rustls::{ServerConfig, Certificate, PrivateKey, ServerConnection};
use mio::{net::{TcpListener, TcpStream}, Token, Registry, Interest, event::Event, Events, Poll};
use super::tlsconfig::server_config;

#[derive(Resource)]
pub struct AuthenticatorServer {
    // The receiver will only be read by one system, so this is probably fine.
    pub receiver: Arc<Mutex<Receiver<AuthenticatorResponse>>>,
}

pub enum AuthenticatorResponse {
    ClientAccepted,
}

#[must_use]
pub fn start_auth_server(
    address: impl Into<SocketAddr>,
    certificates: Vec<Certificate>,
    private_key: PrivateKey,
) -> AuthenticatorServer {
    // Establish channel between auth and main thread
    let channel = mpsc::channel();
    let address = address.into();

    // Start thread
    thread::spawn(move || {
        // Setup inter-thread channel and server struct
        let sender = channel.0;
        let mut server = AuthenticatorServerInternal {
            listener: TcpListener::bind(address)
                .expect(&format!("Auth server could not bind to {:?}", address)),
            connections: HashMap::new(),
            next_id: 2,
            tls_config: server_config(
                certificates,
                private_key
            ),
        };

        // mio types
        let mut poll = Poll::new().unwrap();
        poll
            .registry()
            .register(&mut server.listener, LISTENER, Interest::READABLE)
            .unwrap();
        let mut events = Events::with_capacity(256);

        loop {
            poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    LISTENER => { server.accept(poll.registry()).expect("Error accepting socket"); }
                    _ => server.connection_event(poll.registry(), event)
                }
            }
        }
    });

    AuthenticatorServer {
        receiver: Arc::new(Mutex::new(channel.1))
    }
}

// Code from here on is largely repurposed from the following code by rustls, licensed under the MIT License.
// https://github.com/rustls/rustls/blob/3d121b9d6254a4326a9b92a1c40cb002a84f8188/examples/src/bin/tlsserver-mio.rs

const LISTENER: Token = Token(0);

pub(super) struct AuthenticatorServerInternal {
    listener: TcpListener,
    connections: HashMap<Token, OpenConnection>,
    next_id: usize,
    tls_config: Arc<ServerConfig>,
}

impl AuthenticatorServerInternal {
    fn accept(&mut self, registry: &Registry) -> Result<(), io::Error> {
        loop {
            match self.listener.accept() {
                Ok((socket, addr)) => {
                    debug!("Accepting connection from {:?}", addr);

                    let tls_conn = ServerConnection::new(Arc::clone(&self.tls_config)).unwrap();
                    let token = Token(self.next_id);
                    self.next_id += 1;

                    let mut connection = OpenConnection::new(
                        socket,
                        token,
                        tls_conn,
                        ConnectionState::Connected
                    );
                    connection.register(registry);

                },
                Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => return Ok(()),
                Err(err) => {
                    error!("Error while accepting TCP/TLS connection: {}", err);
                    return Err(err)
                }
            }
        }
    }

    fn connection_event(&mut self, registry: &Registry, event: &Event) {
        let token = event.token();
        if self.connections.contains_key(&token) {
            self.connections.get_mut(&token).unwrap().ready(registry, event.clone());
            if self.connections[&token].state() == ConnectionState::Closed {
                self.connections.remove(&token);
            }
        }
    }
}

pub(super) struct OpenConnection {
    socket: TcpStream,
    token: Token,
    connection: ServerConnection,
    state: ConnectionState,
}

impl OpenConnection {
    pub fn new(
        socket: TcpStream,
        token: Token,
        connection: ServerConnection,
        state: ConnectionState,
    ) -> Self {
        Self {
            socket,
            token,
            connection,
            state,
        }
    }

    pub fn ready(
        &mut self,
        registry: &Registry,
        ev: Event,
    ) {
        if ev.is_readable() { self.do_tls_read(); }
        if ev.is_writable() { self.do_tls_write_and_handle_error(); }
        
        match self.state {
            ConnectionState::Connected => {
                self.reregister(registry);
            },
            ConnectionState::Closing => {
                let _ = self.socket.shutdown(Shutdown::Both);
                self.state = ConnectionState::Closed;
                self.deregister(registry);
            },
            ConnectionState::Closed => {},
        }
    }

    pub fn register(
        &mut self,
        registry: &Registry,
    ) {
        let event_set = self.event_set();
        registry.register(&mut self.socket, self.token, event_set).unwrap();
    }

    pub fn reregister(&mut self, registry: &Registry) {
        let event_set = self.event_set();
        registry.reregister(&mut self.socket, self.token, event_set).unwrap();
    }

    pub fn deregister(&mut self, registry: &Registry) {
        registry.deregister(&mut self.socket).unwrap();
    }

    pub fn event_set(&self) -> Interest {
        let rd = self.connection.wants_read();
        let wr = self.connection.wants_write();

        match (rd, wr) {
            (true, true) => { Interest::READABLE | Interest::WRITABLE },
            (false, true) => { Interest::WRITABLE },
            _ => { Interest::READABLE },
        }
    }

    pub fn do_tls_read(&mut self) {
        match self.connection.read_tls(&mut self.socket) {
            Err(err) => {
                if let io::ErrorKind::WouldBlock = err.kind() { return; }
                error!("TLS server read error: {:?}", err);
                self.state = ConnectionState::Closing;
                return;
            },
            Ok(0) => {
                debug!("TLS server end of field");
                self.state = ConnectionState::Closing;
                return;
            }
            Ok(_) => {}
        }

        if let Err(err) = self.connection.process_new_packets() {
            error!("Cannot process packet: {:?}", err);
        }
    }

    pub fn tls_write(&mut self) -> io::Result<usize> {
        self.connection.write_tls(&mut self.socket)
    }

    pub fn do_tls_write_and_handle_error(&mut self) {
        let rc = self.tls_write();
        if rc.is_err() {
            error!("Write failed: {:?}", rc.unwrap_err());
            self.state = ConnectionState::Closing;
        }
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ConnectionState {
    Connected,
    Closing,
    Closed,
}