use std::{sync::{mpsc::{self, Receiver}, Arc, Mutex}, thread, collections::HashMap, net::SocketAddr, io};
use bevy::prelude::{Resource, error, debug};
use bevy_stardust_shared::rustls::{ServerConfig, Certificate, PrivateKey, ServerConnection};
use mio::{net::{TcpListener, TcpStream}, Token, Registry, Interest};
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
        let sender = channel.0;
        let server = AuthenticatorServerInternal {
            listener: TcpListener::bind(address)
                .expect(&format!("Auth server could not bind to {:?}", address)),
            connections: HashMap::new(),
            next_id: 2,
            tls_config: server_config(
                certificates,
                private_key
            ),
        };
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

                    let mut connection = OpenConnection::new(socket, token, tls_conn);
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
}

pub(super) struct OpenConnection {
    socket: TcpStream,
    token: Token,
    connection: ServerConnection,
}

impl OpenConnection {
    pub fn new(
        socket: TcpStream,
        token: Token,
        connection: ServerConnection,
    ) -> Self {
        Self {
            socket,
            token,
            connection
        }
    }

    pub fn register(
        &mut self,
        registry: &Registry,
    ) {
        let event_set = self.event_set();
        registry.register(&mut self.socket, self.token, event_set).unwrap();
        todo!()
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
}

pub(super) enum ConnectionState {
    Connected,
    Closing,
    Closed,
}